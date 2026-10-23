"""Check profile tabs, page banners, and organizer access on the local demo.

    uv run python scripts/check_profile_ui.py --base-url http://127.0.0.1:8444 \
        --browser-executable /usr/bin/google-chrome --screenshots /tmp/arctos-profiles

Uses the shared loopback/demo marker and mutation guard. Image POST/DELETE
responses and deterministic raster fixtures are fulfilled in the browser; no profile or event changes
reach the backend. The only real mutation is signing into a demo account.
"""

from __future__ import annotations

import asyncio
from io import BytesIO
import re
import sys
from urllib.parse import urlsplit

from PIL import Image
from playwright.async_api import Page, Route, async_playwright, expect

from check_appearance_ui import PALETTES, check_contrast, choose, close_menu, isolated_page
from check_mobile_ui import DEMO_USER, TIMEOUT_MS, SmokeCheck, api_json, arguments

BANNER_KEY = "arctos.page-banners.hidden.v1"
INTRODUCTIONS = (
    ("/", "Events", "#event-search", "#event-discovery"),
    ("/leagues", "Leagues", "#league-search", "#page-content"),
    ("/players", "Players", "#player-search", "#page-content"),
    ("/teams", "Teams", "#team-search", "#page-content"),
    ("/stones", "Stones", "#stones-start-pause", "#stones-start-pause"),
)


async def restore_banner(page: Page) -> None:
    await page.locator("#utility-trigger").click()
    await page.get_by_role("button", name="Show page banner", exact=True).click()
    await expect(page.locator("#utility-options")).to_be_hidden()
    await expect(page.locator("#page-banner-heading")).to_be_focused()


async def introductions(check: SmokeCheck, page: Page) -> None:
    """Hiding one introduction must not hide another page's introduction."""
    for path, title, target, focus in INTRODUCTIONS:
        await page.goto(check.base_url + path, wait_until="domcontentloaded")
        banner = page.locator("#page-banner")
        await expect(banner).to_be_visible()
        await expect(page.locator("#page-banner-heading")).to_have_text(title)
        assert await banner.evaluate("el => getComputedStyle(el).textAlign") == "center"
        dismiss = page.locator("#page-banner-dismiss")
        box = await dismiss.bounding_box()
        assert box and box["width"] >= 44 and box["height"] >= 44, box
        if target.endswith("search"):
            await page.locator(target).fill("demo")
        await dismiss.focus()
        await page.keyboard.press("Enter")
        await expect(banner).to_be_hidden()
        await expect(page.locator(focus)).to_be_focused()
        assert await page.evaluate("!['INPUT', 'TEXTAREA'].includes(document.activeElement.tagName)")
        if target.endswith("search"):
            await expect(page.locator(target)).to_have_value("demo")
        await page.reload(wait_until="domcontentloaded")
        await expect(page.locator(target)).to_be_visible()
        await expect(banner).to_be_hidden()
    stored = await page.evaluate("key => JSON.parse(localStorage.getItem(key))", BANNER_KEY)
    assert set(stored) == {title.lower() for _, title, _, _ in INTRODUCTIONS}, stored
    for path, title, target, _focus in INTRODUCTIONS:
        await page.goto(check.base_url + path, wait_until="domcontentloaded")
        await expect(page.locator(target)).to_be_visible()
        await expect(page.locator("#page-banner")).to_be_hidden()
        await restore_banner(page)
        await expect(page.locator("#page-banner")).to_be_visible()
        await page.reload(wait_until="domcontentloaded")
        await expect(page.locator("#page-banner")).to_be_visible()
        await check.check_layout(page, f"introduction-{title.lower()}", 320)
        check.visited += 1


async def legacy_introduction(check: SmokeCheck, page: Page) -> None:
    await page.goto(check.base_url + "/_api/demo-info", wait_until="domcontentloaded")
    await page.evaluate("localStorage.setItem('arctos.introduction.hidden.v1', 'true')")
    await page.goto(check.base_url, wait_until="domcontentloaded")
    await expect(page.locator("#event-search")).to_be_visible()
    await expect(page.locator("#page-banner")).to_be_hidden()
    await page.goto(check.base_url + "/players", wait_until="domcontentloaded")
    await expect(page.locator("#page-banner")).to_be_visible()
    await page.goto(check.base_url, wait_until="domcontentloaded")
    await expect(page.locator("#event-search")).to_be_visible()
    await restore_banner(page)
    await page.reload(wait_until="domcontentloaded")
    await expect(page.locator("#page-banner")).to_be_visible()


async def selected_profile(page: Page, kind: str, tab: str) -> None:
    await expect(page.locator(f"#{kind}-tab-{tab}")).to_have_attribute("aria-current", "page")
    await expect(page.locator(".profile-section:visible")).to_have_count(1)
    await expect(page.locator(".event-route-tabs [aria-current='page']")).to_have_count(1)


async def profile_composition(page: Page) -> None:
    await expect(page.locator(".profile-cover")).to_be_visible()
    await expect(page.locator(".profile-section.card, .profile-section > .card")).to_have_count(0)
    geometry = await page.locator(".profile-page").evaluate("""profile => {
        const box = selector => {const r = profile.querySelector(selector).getBoundingClientRect();
            return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height};};
        return {cover:box('.profile-cover'),avatar:box('.profile-avatar'),identity:box('.profile-identity'),
            meta:box('.profile-meta'),tabs:box('.event-route-tabs'),
            alignment:getComputedStyle(profile.querySelector('.profile-identity')).textAlign};
    }""")
    cover, avatar, identity = (geometry[key] for key in ("cover", "avatar", "identity"))
    assert abs(cover["width"] / cover["height"] - 3) < 0.02, geometry
    assert avatar["top"] < cover["bottom"] < avatar["bottom"], geometry
    assert abs(avatar["width"] - avatar["height"]) <= 1, geometry
    assert geometry["alignment"] in ("start", "left"), geometry
    assert abs(identity["left"] - avatar["left"]) <= 2, geometry
    assert identity["top"] >= avatar["bottom"] - 1, geometry
    assert geometry["tabs"]["top"] >= geometry["meta"]["bottom"] - 1, geometry


async def public_profiles(check: SmokeCheck, page: Page) -> None:
    for width in (320, 360, 1280):
        await page.set_viewport_size({"width": width, "height": 900})
        for kind in ("player", "team"):
            path = f"/{kind}s/demo-{kind}"
            payload = await api_json(page.context, check.base_url, "/_api" + path)
            await page.goto(check.base_url + path, wait_until="domcontentloaded")
            await selected_profile(page, kind, "info")
            await profile_composition(page)
            await expect(page.get_by_role("link", name="Edit profile", exact=True)).to_have_count(0)
            await expect(page.locator("#profile-banner-file, #profile-banner-remove")).to_have_count(0)
            await expect(page.locator(".profile-event-stat dd")).to_have_text(str(payload["past_event_count"]))
            optional = (
                {"infractions": payload["player_notes"], "injuries": payload["injuries"]}
                if kind == "player"
                else {"notes": payload["team_notes"]}
            )
            if kind == "player":
                assert all(injury["show"] for injury in payload["injuries"]), "Guest profile exposed private injuries"
            for tab, records in optional.items():
                await expect(page.locator(f"#{kind}-tab-{tab}")).to_have_count(1 if records else 0)
                if not records:
                    await page.goto(check.base_url + path + "?tab=" + tab, wait_until="domcontentloaded")
                    await selected_profile(page, kind, "info")
            await page.goto(check.base_url + path + "?tab=history", wait_until="domcontentloaded")
            await selected_profile(page, kind, "history")
            await check.profile_rows_fit(page, ".profile-history-table")
            await page.locator(f"#{kind}-tab-info").click()
            await selected_profile(page, kind, "info")
            await page.go_back()
            await selected_profile(page, kind, "history")
            await page.go_forward()
            await selected_profile(page, kind, "info")
            links = page.locator(".event-route-tabs a")
            await links.first.focus()
            for index in range(1, await links.count()):
                await page.keyboard.press("Tab")
                await expect(links.nth(index)).to_be_focused()
            controls = await links.evaluate_all(
                "els => els.map(el => {const r=el.getBoundingClientRect();return {width:r.width,height:r.height};})"
            )
            assert all(c["width"] >= 44 and c["height"] >= 44 for c in controls), controls
            await check.check_layout(page, f"{kind}-public-tabs", width)
            check.visited += 1
    await page.set_viewport_size({"width": 360, "height": 900})


async def detail_banners(check: SmokeCheck, page: Page) -> None:
    """Legacy hidden entity keys cannot remove identity or metadata from details."""
    await page.goto(check.base_url + "/_api/demo-info", wait_until="domcontentloaded")
    for kind, path, next_path, entity_key in (
        ("player", "/players/demo-player", "/players/demo-player?tab=history", "player:demo-player"),
        ("team", "/teams/demo-team", "/teams/demo-team?tab=history", "team:demo-team"),
        ("event", "/demo-open", "/demo-open/results", "event:demo-open"),
        ("league", "/leagues/demo-coast-league", "/leagues/demo-coast-league/results", "league:demo-coast-league"),
    ):
        await page.evaluate(
            "([key, entity]) => localStorage.setItem(key, JSON.stringify(['events', entity]))",
            [BANNER_KEY, entity_key],
        )
        await page.goto(check.base_url + path, wait_until="domcontentloaded")
        heading = page.locator("#page-banner-heading")
        await expect(heading).to_be_visible()
        title = await heading.inner_text()

        async def visible_detail() -> None:
            banner = page.locator("#page-banner")
            await expect(banner).to_be_visible()
            await expect(banner).not_to_have_class(re.compile(r"\bis-collapsed\b"))
            await expect(heading).to_have_text(title)
            await expect(page.locator("#page-banner-dismiss")).to_have_count(0)
            if kind in ("event", "league"):
                await expect(page.locator(".event-hero-badges")).to_be_visible()
                await expect(page.locator(".event-hero-meta")).to_be_visible()
                await expect(page.locator(".event-hero-actions .btn").first).to_be_visible()
            else:
                await profile_composition(page)
                await expect(page.locator(".profile-event-stat")).to_be_visible()
            await expect(page.locator(".event-route-tabs")).to_be_visible()
            await page.locator("#utility-trigger").click()
            await expect(page.locator("#utility-options")).to_be_visible()
            await expect(page.locator("#page-banner-restore")).to_have_count(0)
            await page.keyboard.press("Escape")
            await expect(page.locator("#utility-options")).to_be_hidden()

        await visible_detail()
        await page.goto(check.base_url + next_path, wait_until="domcontentloaded")
        await visible_detail()
        await page.locator(f"#{kind}-tab-info").click()
        await expect(page.locator(f"#{kind}-tab-info")).to_have_attribute("aria-current", "page")
        await visible_detail()
        await page.go_back(wait_until="domcontentloaded")
        await expect(page).to_have_url(check.base_url + next_path)
        await visible_detail()
        await page.go_forward(wait_until="domcontentloaded")
        await expect(page.locator(f"#{kind}-tab-info")).to_have_attribute("aria-current", "page")
        await visible_detail()
        await page.reload(wait_until="domcontentloaded")
        await visible_detail()
        await check.check_layout(page, f"{kind}-banner-always-visible", 360)
        check.visited += 1


def sample_png(width: int, height: int) -> bytes:
    """A deterministic color gradient makes crop movement observable in saved pixels."""
    image = Image.new("RGB", (width, height))
    image.putdata([(x * 255 // (width - 1), y * 255 // (height - 1), 80) for y in range(height) for x in range(width)])
    output = BytesIO()
    image.save(output, format="PNG")
    return output.getvalue()


PNG = sample_png(1200, 800)
SMALL_PNG = sample_png(120, 80)
URL_MONITOR = """window.cropUrls = {created: [], revoked: []};
const create = URL.createObjectURL.bind(URL), revoke = URL.revokeObjectURL.bind(URL);
URL.createObjectURL = value => {const url = create(value); cropUrls.created.push(url); return url;};
URL.revokeObjectURL = url => {cropUrls.revoked.push(url); revoke(url);};"""


async def adjust_crop(page: Page, crop_id: str) -> None:
    viewport = page.locator("#" + crop_id)
    zoom = page.locator("#" + crop_id + "-zoom")
    await zoom.focus()
    await page.keyboard.press("End")
    await expect(zoom).to_have_value("3")
    await viewport.scroll_into_view_if_needed()
    image = viewport.locator("img")
    style = await image.get_attribute("style")
    box = await viewport.bounding_box()
    assert box
    x, y = box["x"] + box["width"] / 2, box["y"] + box["height"] / 2
    await page.mouse.move(x, y)
    await page.mouse.down()
    await page.mouse.move(x + 30, y + 12, steps=5)
    await page.mouse.up()
    await expect(image).not_to_have_attribute("style", style)
    if crop_id == "image-crop-avatar":
        style = await image.get_attribute("style")
        cdp = await page.context.new_cdp_session(page)
        touch_active = False
        try:
            point = {"x": x, "y": y, "id": 1}
            await cdp.send("Input.dispatchTouchEvent", {"type": "touchStart", "touchPoints": [point]})
            touch_active = True
            await cdp.send(
                "Input.dispatchTouchEvent", {"type": "touchMove", "touchPoints": [{**point, "x": x - 20, "y": y - 15}]}
            )
            await cdp.send("Input.dispatchTouchEvent", {"type": "touchEnd", "touchPoints": []})
            touch_active = False
            await expect(image).not_to_have_attribute("style", style)
        finally:
            if touch_active:
                await cdp.send("Input.dispatchTouchEvent", {"type": "touchCancel", "touchPoints": []})
            await cdp.detach()
    editor = viewport.locator("..")
    await editor.locator("summary").click()
    horizontal = page.locator("#" + crop_id + "-horizontal")
    await horizontal.focus()
    await page.keyboard.press("Home")
    await expect(horizontal).to_have_value("0")
    await editor.get_by_role("button", name="Center", exact=True).click()
    await expect(horizontal).to_have_value("50")
    await expect(page.locator("#" + crop_id + "-vertical")).to_have_value("50")
    await horizontal.focus()
    await page.keyboard.press("End")
    await expect(horizontal).to_have_value("100")


async def image_upload(check: SmokeCheck, page: Page, kind: str, avatar: bool = False) -> None:
    profile = kind in ("player", "team")
    noun = "photo" if avatar else "banner"
    field = "profile_photo" if avatar else "banner_photo"
    owner = f"demo-{kind}" if profile else DEMO_USER
    await check.login(page, owner)
    path = f"/{kind}s/{owner}" if profile else "/demo-open" if kind == "tournament" else "/leagues/demo-coast-league"
    api_path = "/_api" + ("/tournaments" if kind == "tournament" else "") + path
    endpoint = api_path + ("/profile-photo" if avatar else "/banner-photo")
    edit_path = path + ("/edit" if profile else "/settings")
    data = await api_json(page.context, check.base_url, api_path)
    current = "smoke-profile/old.png"
    source = SMALL_PNG if avatar else PNG
    error = "Smoke check: image upload failed. Existing image kept."
    succeed = False
    writes, images = [], []
    gate = asyncio.Event()
    request_seen = asyncio.Event()
    check.mock_gate = gate

    async def fixture(route: Route) -> None:
        nonlocal current
        request, parsed = route.request, urlsplit(route.request.url)
        if parsed.netloc != check.origin:
            await route.fallback()
        elif request.method == "GET" and parsed.path.startswith("/static/smoke-profile/"):
            await route.fulfill(
                body=images[-1] if images and parsed.path.endswith("new.png") else source, content_type="image/png"
            )
        elif request.method == "GET" and parsed.path == api_path:
            await route.fulfill(json={**data, kind: {**data[kind], field: current}})
        elif parsed.path == endpoint and request.method in ("POST", "DELETE"):
            writes.append(request.method)
            if request.method == "POST":
                images.append(request.post_data_buffer)
                request_seen.set()
                await gate.wait()
            if request.method == "DELETE":
                current = None
                await route.fulfill(json={"success": True})
            elif not succeed:
                await route.fulfill(json={"success": False, "error": error})
            else:
                current = "smoke-profile/new.png"
                await route.fulfill(json={"success": True, "path": current})
        else:
            await route.fallback()

    await page.add_init_script(URL_MONITOR)
    await page.context.route("**/*", fixture)
    try:
        await page.goto(check.base_url + edit_path, wait_until="domcontentloaded")
        upload = page.locator("#profile-photo-file" if avatar else "#profile-banner-file")
        await expect(upload).to_be_visible()
        palette, mode = {
            ("player", False): ("california", "light"),
            ("team", False): ("field", "dark"),
            ("tournament", False): ("neutral", "light"),
            ("league", False): ("california", "dark"),
            ("player", True): ("field", "light"),
            ("team", True): ("neutral", "dark"),
        }[kind, avatar]
        await choose(page, palette, mode)
        await close_menu(page)
        widget = page.get_by_role("region", name="Profile photo" if avatar else "Banner image", exact=True)
        crop_id = "image-crop-avatar" if avatar else "image-crop-banner"
        crop = page.locator("#" + crop_id)
        save_form = page.locator("main form button[type=submit]").first
        save_crop = widget.get_by_role("button", name="Save " + noun, exact=True)
        draft = page.locator("#about") if kind == "league" else page.locator('main form input[type="text"]').first
        await draft.fill("Unsaved local details")
        registration_open = None
        if not profile:
            await page.locator("#team_reg_fee").fill("123.45")
            registration_open = not await page.locator("#team_registration_open").is_checked()
            await page.locator('label[for="team_registration_open"]').click()

        async def draft_retained() -> None:
            await expect(draft).to_have_value("Unsaved local details")
            if not profile:
                await expect(page.locator("#team_reg_fee")).to_have_value("123.45")
                await expect(page.locator("#team_registration_open")).to_be_checked(checked=registration_open)

        preview = widget.locator(".photo-upload-preview" if avatar else ".banner-upload-preview")
        await expect(preview).to_have_attribute("src", check.base_url + "/static/" + current)
        if kind == "player" and not avatar:
            for name, body, message in (
                ("broken.png", b"not an image", "valid JPEG"),
                ("large.png", PNG + bytes(8 * 1024 * 1024), "8 MB"),
            ):
                await upload.set_input_files({"name": name, "mimeType": "image/png", "buffer": body})
                await expect(widget.get_by_role("alert")).to_contain_text(message)
                await expect(crop).to_have_count(0)
                await expect(save_form).to_be_enabled()
            assert not writes

        await upload.set_input_files({"name": "cancel.png", "mimeType": "image/png", "buffer": source})
        await expect(crop).to_be_visible()
        cancelled_url = await crop.locator("img").get_attribute("src")
        await expect(page.locator("#" + crop_id + "-zoom")).to_be_focused()
        await expect(save_form).to_be_disabled()
        await adjust_crop(page, crop_id)
        assert not writes, "Choosing or adjusting a crop uploaded it before Save"
        await widget.get_by_role("button", name="Cancel", exact=True).click()
        await expect(crop).to_have_count(0)
        await expect(upload).to_be_focused()
        await expect(save_form).to_be_enabled()
        assert await page.evaluate("url => cropUrls.revoked.includes(url)", cancelled_url)
        await draft_retained()

        await upload.set_input_files({"name": "save.png", "mimeType": "image/png", "buffer": source})
        await expect(crop).to_be_visible()
        saved_url = await crop.locator("img").get_attribute("src")
        await adjust_crop(page, crop_id)
        positions = await widget.locator("input[type=range]").evaluate_all("els => els.map(el => el.value)")
        await check_contrast(
            page,
            ".image-crop-editor, .image-crop-editor .form-label, .image-crop-actions .btn, .image-upload > .form-text",
        )
        await check.check_layout(page, f"{kind}-{noun}-crop-{palette}-{mode}", 320)
        check.visited += 1
        await save_crop.click()
        await asyncio.wait_for(request_seen.wait(), 10)
        await expect(save_form).to_be_disabled()
        gate.set()
        await expect(widget.get_by_role("alert")).to_have_text(error)
        await expect(crop).to_be_visible()
        await expect(preview).to_have_attribute("src", check.base_url + "/static/" + current)
        assert await widget.locator("input[type=range]").evaluate_all("els => els.map(el => el.value)") == positions
        await draft_retained()
        succeed = True
        gate.clear()
        request_seen.clear()
        await save_crop.click()
        await asyncio.wait_for(request_seen.wait(), 10)
        gate.set()
        await expect(widget.get_by_role("status")).to_have_text(noun.title() + " updated.")
        await expect(crop).to_have_count(0)
        await expect(upload).to_be_focused()
        await expect(save_form).to_be_enabled()
        await draft_retained()
        assert images[0] == images[1], "Retry changed the user's selected crop"
        with Image.open(BytesIO(images[-1])) as result, Image.open(BytesIO(source)) as original:
            assert result.format == "PNG"
            assert result.width == result.height * (1 if avatar else 3), result.size
            assert result.width <= (512 if avatar else 1500) and result.height <= (512 if avatar else 500), result.size
            assert result.width <= original.width / 3 + 1 and result.height <= original.height / 3 + 1, (
                "Zoomed crop was upscaled"
            )
            center = result.convert("RGB").getpixel((result.width // 2, result.height // 2))
            original_center = original.getpixel((original.width // 2, original.height // 2))
            assert sum(abs(a - b) for a, b in zip(center, original_center)) > 20, "Saved crop ignored position controls"
        assert await page.evaluate("url => cropUrls.revoked.includes(url)", saved_url)
        viewer = await page.context.new_page()
        check.observe(viewer)
        try:
            await viewer.goto(check.base_url + path, wait_until="domcontentloaded")
            image = ".profile-avatar" if avatar else ".profile-cover-image" if profile else ".event-banner-image img"
            await expect(viewer.locator(image)).to_have_attribute(
                "src", check.base_url + "/static/smoke-profile/new.png"
            )
            await viewer.locator(image).evaluate("img => img.decode()")
            await check.check_layout(viewer, f"{kind}-uploaded-{noun}", 320)
            check.visited += 1
        finally:
            await viewer.close()
        await widget.get_by_role("button", name="Remove " + noun, exact=True).click()
        await expect(widget.get_by_role("status")).to_have_text(noun.title() + " removed.")
        await expect(preview).to_have_count(0)
        await draft_retained()
        assert writes == ["POST", "POST", "DELETE"], writes
        await check.check_layout(page, f"{kind}-{noun}-upload", 320)
        check.visited += 1
        await upload.set_input_files({"name": "leave.png", "mimeType": "image/png", "buffer": source})
        await expect(crop).to_be_visible()
        await (
            page.get_by_role("navigation", name="Main navigation")
            .get_by_role("link", name="Events", exact=True)
            .click()
        )
        await expect(page.locator("#event-search")).to_be_visible()
        assert await page.evaluate("cropUrls.created.every(url => cropUrls.revoked.includes(url))"), (
            "Crop object URL leaked on route change"
        )
        assert writes == ["POST", "POST", "DELETE"], "Leaving the crop submitted it"
    finally:
        gate.set()
        await page.context.unroute("**/*", fixture)


async def organizer_tools(check: SmokeCheck, page: Page) -> None:
    paths = ("/demo-open", "/leagues/demo-coast-league")
    for path in paths:
        await page.goto(check.base_url + path, wait_until="domcontentloaded")
        await expect(page.locator("#page-banner-heading")).to_be_visible()
        await expect(page.locator("#organizer-tools-trigger")).to_have_count(0)
    await check.login(page)
    for path in paths:
        await page.goto(check.base_url + path, wait_until="domcontentloaded")
        trigger = page.locator("#organizer-tools-trigger")
        options = page.locator("#organizer-tools-options")
        await expect(trigger).to_be_visible()
        await expect(options).to_be_hidden()
        await trigger.focus()
        await page.keyboard.press("Enter")
        await expect(trigger).to_be_focused()
        await expect(options).to_be_visible()
        settings = options.get_by_role("link", name="Settings", exact=True)
        await page.keyboard.press("Tab")
        await expect(settings).to_be_focused()
        await expect(settings).to_have_attribute("href", path + "/settings")
        controls = await options.locator("a, button").evaluate_all(
            "els => els.map(el => {const r=el.getBoundingClientRect();return {w:r.width,h:r.height,left:r.left,right:r.right};})"
        )
        assert all(c["w"] >= 44 and c["h"] >= 44 and c["left"] >= -1 and c["right"] <= 321 for c in controls), controls
        await page.keyboard.press("Escape")
        await expect(options).to_be_hidden()
        await expect(trigger).to_be_focused()
        await trigger.click()
        await page.locator(".organizer-tools-dismiss").click(position={"x": 2, "y": 2})
        await expect(options).to_be_hidden()
        await expect(trigger).to_be_focused()
        await expect(page.locator("#page-banner-dismiss")).to_have_count(0)
        await expect(page.locator(".event-hero-meta")).to_be_visible()
        await trigger.click()
        await check.check_layout(page, f"organizer-tools-{'league' if path.startswith('/leagues') else 'event'}", 320)
        check.visited += 1
        await settings.click()
        await expect(page).to_have_url(check.base_url + path + "/settings")
        await expect(page.locator("#profile-banner-file")).to_be_visible()


async def owner_profile_tabs(check: SmokeCheck, page: Page, kind: str) -> None:
    if kind == "team":
        await check.login(page, "demo-team")
        await check.team_profile_tabs(page)
    else:
        await check.player_profile_tabs(page)
    await check.check_layout(page, f"{kind}-owner-tabs", 320)
    check.visited += 1


async def profile_themes(check: SmokeCheck, page: Page) -> None:
    for mode in ("light", "dark"):
        for palette in PALETTES:
            for kind in ("player", "team"):
                await page.goto(check.base_url + f"/{kind}s/demo-{kind}", wait_until="domcontentloaded")
                await selected_profile(page, kind, "info")
                await choose(page, palette, mode)
                await close_menu(page)
                await check_contrast(
                    page,
                    ".profile-section, .event-route-tab, .profile-detail-header h1, .profile-account-name, .profile-meta, .profile-event-stat dt, .profile-event-stat dd",
                )
                await check.check_layout(page, f"{kind}-hero-{palette}-{mode}", 360)
                check.visited += 1


async def run(args) -> int:
    check = SmokeCheck(args.base_url, args.screenshots)
    async with async_playwright() as playwright:
        api = await playwright.request.new_context()
        try:
            response = await api.get(args.base_url + "/_api/demo-info", timeout=TIMEOUT_MS)
            marker = await response.json() if response.ok else {}
            assert marker.get("demo") is True and marker.get("organizer") == DEMO_USER, "Verified local demo required"
        finally:
            await api.dispose()
        launch = {"headless": True, "args": ["--mute-audio"]}
        if args.browser_executable:
            launch["executable_path"] = str(args.browser_executable)
        browser = await playwright.chromium.launch(**launch)
        try:
            for name, width, action in (
                ("page-introductions", 320, introductions),
                ("legacy-introduction", 360, legacy_introduction),
                ("public-profile-tabs", 360, public_profiles),
                ("detail-banners-always-visible", 360, detail_banners),
                ("player-owner-tabs", 320, lambda check, page: owner_profile_tabs(check, page, "player")),
                ("team-owner-tabs", 320, lambda check, page: owner_profile_tabs(check, page, "team")),
                ("player-banner-upload", 320, lambda check, page: image_upload(check, page, "player")),
                ("team-banner-upload", 320, lambda check, page: image_upload(check, page, "team")),
                ("tournament-banner-upload", 320, lambda check, page: image_upload(check, page, "tournament")),
                ("league-banner-upload", 320, lambda check, page: image_upload(check, page, "league")),
                ("player-photo-upload", 320, lambda check, page: image_upload(check, page, "player", True)),
                ("team-photo-upload", 320, lambda check, page: image_upload(check, page, "team", True)),
                ("organizer-tools", 320, organizer_tools),
                ("profile-theme-matrix", 360, profile_themes),
            ):
                async with isolated_page(check, browser) as page:
                    await page.set_viewport_size({"width": width, "height": 900})
                    await check.workflow(page, name, width, lambda: action(check, page))
        finally:
            await browser.close()
    print(
        f"{check.visited} profile/banner screens; {check.workflows} workflows passed; {len(check.failures)} failures."
    )
    if check.failures:
        print("\n".join(check.failures), file=sys.stderr)
    return 1 if check.failures else 0


if __name__ == "__main__":
    try:
        raise SystemExit(asyncio.run(run(arguments(__doc__))))
    except (AssertionError, RuntimeError) as error:
        print(f"Profile check stopped: {error}", file=sys.stderr)
        raise SystemExit(1)
