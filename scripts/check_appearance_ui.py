"""Check Arctos appearance settings against the explicitly started local demo.

    uv run python scripts/check_appearance_ui.py --base-url http://127.0.0.1:8444 \
        --browser-executable /usr/bin/google-chrome --screenshots /tmp/arctos-appearance

Checks all four palettes in light/dark at phone width, representative custom
surfaces, persistence, live System changes, explicit overrides, cross-tab
changes/removal, blocked browser storage, keyboard focus, and overlay isolation.
Shares the demo marker, loopback restriction, asset checks, and mutation guard
with check_mobile_ui.py. No server data is changed except signing into the demo.
"""

from __future__ import annotations

import asyncio
import json
import sys

from playwright.async_api import Page, async_playwright, expect

from check_mobile_ui import DEMO_USER, TIMEOUT_MS, Screen, SmokeCheck, api_json, arguments

STORAGE_KEY = "arctos.appearance.v1"
WIDTH = 360
PALETTES = ("forest", "ocean", "plum", "ember")


async def preference(page: Page, palette: str, resolved_mode: str) -> None:
    app = page.locator(".arctos-app")
    await expect(app).to_have_attribute("data-palette", palette)
    await expect(app).to_have_attribute("data-bs-theme", resolved_mode)


async def choose(page: Page, palette: str, mode: str) -> None:
    trigger = page.get_by_role("button", name="Appearance", exact=True)
    if not await page.locator("#appearance-options").is_visible():
        await trigger.click()
    await page.get_by_role("radio", name=f"{palette.title()} palette", exact=True).check()
    await page.get_by_role("radio", name=f"{mode.title()} mode", exact=True).check()


async def close_menu(page: Page) -> None:
    await page.keyboard.press("Escape")
    await expect(page.locator("#appearance-options")).to_be_hidden()
    await expect(page.locator("#appearance-trigger")).to_be_focused()


async def check_contrast(page: Page, selector: str, minimum: float = 4.5) -> None:
    """Check actual text against its composited ancestor backgrounds."""
    await expect(page.locator(selector).first).to_be_visible()
    failures = await page.locator(selector).evaluate_all(
        """(elements, minimum) => {
        const rgba = value => {
            const values = value.match(/[\\d.]+/g)?.map(Number);
            return values && values.length >= 3 ? [...values.slice(0, 3), values[3] ?? 1] : [0, 0, 0, 0];
        };
        const luminance = rgb => rgb.slice(0, 3).map(v => {
            v /= 255; return v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4;
        }).reduce((sum, v, i) => sum + v * [.2126, .7152, .0722][i], 0);
        return elements.filter(el => el.getBoundingClientRect().height > 0).flatMap(el => {
            let bg = [255, 255, 255], ancestors = [], current = el;
            while (current) { ancestors.unshift(current); current = current.parentElement; }
            for (const ancestor of ancestors) {
                const color = rgba(getComputedStyle(ancestor).backgroundColor);
                bg = bg.map((v, i) => color[i] * color[3] + v * (1 - color[3]));
            }
            const fg = rgba(getComputedStyle(el).color).slice(0, 3);
            const a = luminance(fg), b = luminance(bg);
            const ratio = (Math.max(a, b) + .05) / (Math.min(a, b) + .05);
            return ratio + .01 >= minimum ? [] : [{selector: el.className, fg, bg, ratio}];
        });
    }""",
        minimum,
    )
    assert not failures, f"Text contrast below {minimum}: {failures}"


async def palette_matrix(check: SmokeCheck, page: Page) -> None:
    accents: dict[str, set[str]] = {"light": set(), "dark": set()}
    for mode in ("light", "dark"):
        for palette in PALETTES:
            await page.goto(check.base_url, wait_until="domcontentloaded")
            await choose(page, palette, mode)
            await preference(page, palette, mode)
            await check.check_layout(page, f"appearance-menu-{palette}-{mode}", WIDTH)
            await close_menu(page)
            await check.check_layout(page, f"appearance-home-{palette}-{mode}", WIDTH)
            await check_contrast(
                page,
                ".arctos-discovery-hero > p, .arctos-hero-actions .btn-primary, .arctos-event-card .btn, .arctos-card-title a",
            )
            accents[mode].add(
                await page.locator(".arctos-app").evaluate(
                    "el => getComputedStyle(el).getPropertyValue('--ui-accent').trim()"
                )
            )
            check.visited += 1
    assert all(len(colors) == 4 for colors in accents.values()), f"Palettes did not produce distinct accents: {accents}"


async def keyboard_and_persistence(check: SmokeCheck, page: Page) -> None:
    await page.goto(check.base_url, wait_until="domcontentloaded")
    await choose(page, "forest", "light")
    await close_menu(page)
    trigger = page.get_by_role("button", name="Appearance", exact=True)
    await trigger.focus()
    await page.keyboard.press("Enter")
    forest = page.get_by_role("radio", name="Forest palette", exact=True)
    await expect(forest).to_be_focused()
    await page.keyboard.press("ArrowRight")
    await expect(page.get_by_role("radio", name="Ocean palette", exact=True)).to_be_checked()
    await page.get_by_role("radio", name="Dark mode", exact=True).check()
    await close_menu(page)
    await preference(page, "ocean", "dark")
    saved = await page.evaluate("key => JSON.parse(localStorage.getItem(key))", STORAGE_KEY)
    assert saved == {"palette": "ocean", "mode": "dark"}, f"Unexpected stored preference: {saved}"
    await page.reload(wait_until="domcontentloaded")
    await preference(page, "ocean", "dark")
    await page.get_by_role("navigation", name="Main navigation").get_by_role("link", name="Teams", exact=True).click()
    await expect(page).to_have_url(check.base_url + "/teams")
    await preference(page, "ocean", "dark")


async def system_changes(check: SmokeCheck, page: Page) -> None:
    await page.goto(check.base_url, wait_until="domcontentloaded")
    await page.emulate_media(color_scheme="light")
    await choose(page, "plum", "system")
    await close_menu(page)
    await preference(page, "plum", "light")
    await page.emulate_media(color_scheme="dark")
    await preference(page, "plum", "dark")
    await page.emulate_media(color_scheme="light")
    await preference(page, "plum", "light")
    await choose(page, "plum", "dark")
    await close_menu(page)
    await preference(page, "plum", "dark")
    await page.emulate_media(color_scheme="dark")
    await page.emulate_media(color_scheme="light")
    await preference(page, "plum", "dark")
    await choose(page, "plum", "light")
    await close_menu(page)
    await page.emulate_media(color_scheme="dark")
    await preference(page, "plum", "light")


async def cross_tab(check: SmokeCheck, page: Page) -> None:
    other = await page.context.new_page()
    check.observe(other)
    try:
        await other.goto(check.base_url, wait_until="domcontentloaded")
        await choose(other, "ember", "dark")
        await preference(page, "ember", "dark")
        await other.evaluate("key => localStorage.removeItem(key)", STORAGE_KEY)
        await preference(page, "forest", "dark")  # System follows page's dark media setting.
        await page.emulate_media(color_scheme="light")
        await preference(page, "forest", "light")
    finally:
        await other.close()


async def custom_surfaces(check: SmokeCheck, page: Page) -> None:
    await check.login(page)
    sidecomps = await api_json(page.context, check.base_url, "/_api/demo-open/sidecomps")
    comp_id = next(comp["id"] for comp in sidecomps if comp["name"] == "Sideline Dueling — Demo")
    for mode in ("light", "dark"):
        await page.goto(check.base_url, wait_until="domcontentloaded")
        await choose(page, "ocean", mode)
        await close_menu(page)
        for name, path, ready, contrast in (
            (
                "schedule",
                "/demo-open/schedule?view=field",
                ".schedule-timeline-event",
                ".schedule-timeline-event-name, .schedule-timeline-status-tag--corner",
            ),
            (
                "bracket",
                "/demo-open/bracket",
                "#bracket-canvas-wrap",
                ".bracket-canvas-wrap, .bracket-empty-hint, .bracket-match-name, .bracket-text",
            ),
            (
                "scoring",
                f"/demo-open/sidecomps/{comp_id}/enter-results",
                ".sc-numpad",
                ".sc-num-btn, .sc-display-name, .sc-meta",
            ),
        ):
            await check.visit(page, Screen(f"appearance-{name}-ocean-{mode}", path, ready), WIDTH)
            await preference(page, "ocean", mode)
            await check_contrast(page, contrast)
        await page.goto(check.base_url, wait_until="domcontentloaded")
        await expect(page.locator("#event-search")).to_be_visible()


async def overlay_isolation(check: SmokeCheck, page: Page) -> None:
    for path, ready in (
        ("/demo-open/scoreboard?field=Field%20A", ".scoreboard-container"),
        ("/scoreboard-render", None),
    ):
        await page.goto(check.base_url + path, wait_until="domcontentloaded")
        if ready:
            await expect(page.locator(ready)).to_be_visible()
        else:
            await page.wait_for_function("() => typeof window.arctosRenderScoreboard === 'function'")
        await expect(page.locator(".arctos-app, #appearance-trigger, [data-palette], [data-bs-theme]")).to_have_count(0)
    # Return to the regular shell for shared layout/error checks.
    await page.goto(check.base_url, wait_until="domcontentloaded")
    await expect(page.locator("#event-search")).to_be_visible()


async def storage_disabled(check: SmokeCheck, browser) -> None:
    context = await browser.new_context(
        viewport={"width": WIDTH, "height": 900},
        color_scheme="light",
        has_touch=True,
        is_mobile=True,
        reduced_motion="reduce",
    )
    context.set_default_timeout(TIMEOUT_MS)
    await context.add_init_script("""Object.defineProperty(window, 'localStorage', {
        configurable: true, get() { throw new DOMException('Storage blocked by smoke check', 'SecurityError'); }
    });""")
    await context.route("**/*", check.guard_requests)
    page = await context.new_page()
    check.authenticated = False
    check.observe(page)
    try:
        await page.goto(check.base_url, wait_until="domcontentloaded")
        await preference(page, "forest", "light")
        await choose(page, "ember", "dark")
        await expect(page.locator(".appearance-note")).to_contain_text("storage is unavailable")
        await preference(page, "ember", "dark")
        await close_menu(page)
        await check.check_layout(page, "appearance-storage-unavailable", WIDTH)
        await page.reload(wait_until="domcontentloaded")
        await preference(page, "forest", "light")
    finally:
        await context.close()


async def run(args) -> int:
    check = SmokeCheck(args.base_url, args.screenshots)
    async with async_playwright() as playwright:
        preflight = await playwright.request.new_context()
        try:
            response = await preflight.get(args.base_url + "/_api/demo-info", timeout=TIMEOUT_MS)
            marker = await response.json() if response.ok else {}
            assert marker.get("demo") is True and marker.get("organizer") == DEMO_USER, "Verified local demo required"
        finally:
            await preflight.dispose()
        launch = {"headless": True}
        if args.browser_executable:
            launch["executable_path"] = str(args.browser_executable)
        browser = await playwright.chromium.launch(**launch)
        context = await browser.new_context(
            viewport={"width": WIDTH, "height": 900},
            color_scheme="light",
            has_touch=True,
            is_mobile=True,
            reduced_motion="reduce",
        )
        context.set_default_timeout(TIMEOUT_MS)
        await context.route("**/*", check.guard_requests)
        page = await context.new_page()
        check.observe(page)
        try:
            await page.goto(args.base_url, wait_until="domcontentloaded")
            await preference(page, "forest", "light")
            for name, action in (
                ("palette-mode-matrix", lambda: palette_matrix(check, page)),
                ("keyboard-and-persistence", lambda: keyboard_and_persistence(check, page)),
                ("system-and-overrides", lambda: system_changes(check, page)),
                ("cross-tab-and-removal", lambda: cross_tab(check, page)),
                ("custom-surfaces", lambda: custom_surfaces(check, page)),
                ("overlay-isolation", lambda: overlay_isolation(check, page)),
                ("storage-unavailable", lambda: storage_disabled(check, browser)),
            ):
                await check.workflow(page, f"appearance-{name}", WIDTH, action)
        finally:
            await context.close()
            await browser.close()
    print(
        f"\n{check.visited} appearance screen checks; {check.workflows} workflows passed; {len(check.failures)} failures."
    )
    if check.failures:
        print(json.dumps(check.failures, indent=2), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(asyncio.run(run(arguments(__doc__))))
    except KeyboardInterrupt:
        raise SystemExit(130) from None
    except Exception as error:
        print(f"Appearance check stopped: {error}", file=sys.stderr)
        raise SystemExit(1) from error
