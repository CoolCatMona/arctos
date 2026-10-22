"""Check the running Arctos demo in Chromium at phone, tablet, and desktop widths.

Start ``uv run python -m scripts.dev_demo`` and ``just frontend`` first, then:

    uv run python scripts/check_mobile_ui.py
    uv run python scripts/check_mobile_ui.py --base-url http://127.0.0.1:8443 \
        --browser-executable /usr/bin/google-chrome --screenshots /tmp/arctos-ui

This explicitly invoked browser check is separate from backend pytest. It refuses
non-loopback URLs and requires the demo-only /_api/demo-info marker before login.
All data-changing requests are blocked; the only real POSTs allowed are demo
login and read-only Markdown rendering. A failed finalize request is simulated
in the browser, so fixtures remain unchanged. Match UUIDs and competition IDs
are discovered from the running demo instead of assuming database IDs.
"""

from __future__ import annotations

import argparse
import asyncio
from dataclasses import dataclass
import json
from pathlib import Path
import re
import sys
from urllib.parse import urlsplit

from playwright.async_api import BrowserContext, Page, Route, async_playwright, expect

WIDTHS = (360, 390, 768, 1280)
TIMEOUT_MS = 20_000
DEMO_USER = "demo-organizer"
DEMO_PASSWORD = "arctos-demo-only"
MOCK_ERROR = "Smoke check: simulated connection failure. Your input is safe."


@dataclass(frozen=True)
class Screen:
    name: str
    path: str
    ready: str = "main h1, main h2, main h3"


PUBLIC_SCREENS = (
    Screen("home", "/", "#event-search"),
    Screen("login", "/login", "#username"),
    Screen("register-choice", "/register"),
    Screen("register-player", "/register/player", "#confirm_password"),
    Screen("register-team", "/register/team", "#confirm_password"),
    Screen("leagues", "/leagues"),
    Screen("players", "/players"),
    Screen("teams", "/teams"),
    Screen("event-live", "/demo-open"),
    Screen("event-upcoming", "/demo-summer-cup"),
    Screen("event-past", "/demo-spring-cup"),
    Screen("league", "/leagues/demo-coast-league"),
    Screen("schedule", "/demo-open/schedule", ".schedule-keyboard-focus"),
    Screen("results", "/demo-open/results"),
    Screen("past-results", "/demo-spring-cup/results"),
    Screen("league-results", "/leagues/demo-coast-league/results"),
    Screen("player-profile", "/players/demo-player"),
    Screen("team-profile", "/teams/demo-team"),
    Screen("stones", "/stones", ".stones-page"),
    Screen("about", "/about"),
    Screen("docs", "/docs.md"),
)

AUTH_SCREENS = (
    Screen("create-event", "/create-event", "main .card"),
    Screen("new-tournament", "/new-tournament", "#name"),
    Screen("new-league", "/leagues/new", "#league_name"),
    Screen("league-new-event", "/leagues/demo-coast-league/new-tournament", "#name"),
    Screen("bracket", "/demo-open/bracket", "#bracket-keyboard-focus"),
    Screen("profile-edit", "/players/demo-organizer/edit", "main form"),
    Screen("event-settings", "/demo-open/settings", "#tournament-settings-form"),
    Screen("event-manage", "/demo-open/manage", "main table"),
    Screen("event-footage", "/demo-open/manage-footage"),
    Screen("event-register", "/demo-summer-cup/register"),
    Screen("sidecomp-new", "/demo-open/sidecomps/new", "main form"),
    Screen("schedule-editor", "/demo-open/schedule/edit", ".schedule-editor-split"),
    Screen("league-settings", "/leagues/demo-coast-league/settings", "main form"),
    Screen("league-manage", "/leagues/demo-coast-league/manage", "main table"),
    Screen("league-register", "/leagues/demo-coast-league/register"),
    Screen("developer-apps", "/developers/apps"),
)

# Only document overflow is a failure: schedules, tables, and canvases may have
# intentional scrolling/oversized descendants clipped by their own container.
LAYOUT_DIAGNOSTICS = """() => {
    const width = document.documentElement.clientWidth;
    const documentWidth = Math.max(document.documentElement.scrollWidth, document.body.scrollWidth);
    const escaped = [...document.querySelectorAll('body *')].filter(el => {
        const r = el.getBoundingClientRect();
        if (!r.width || !r.height || (r.left >= -1 && r.right <= width + 1)) return false;
        let parent = el.parentElement;
        while (parent && parent !== document.body) {
            const style = getComputedStyle(parent);
            if (['auto', 'scroll', 'hidden', 'clip'].includes(style.overflowX)) return false;
            parent = parent.parentElement;
        }
        return getComputedStyle(el).position !== 'fixed';
    }).slice(0, 10).map(el => ({
        tag: el.tagName.toLowerCase(), id: el.id, class: String(el.className).slice(0, 120),
        left: Math.round(el.getBoundingClientRect().left),
        right: Math.round(el.getBoundingClientRect().right)
    }));
    return {width, documentWidth, escaped};
}"""


class SmokeCheck:
    def __init__(self, base_url: str, screenshots: Path | None):
        self.base_url = base_url
        self.origin = urlsplit(base_url).netloc
        self.screenshots = screenshots
        self.failures: list[str] = []
        self.page_errors: list[str] = []
        self.network_errors: list[str] = []
        self.blocked_writes: list[str] = []
        self.mock_gate: asyncio.Event | None = None
        self.mock_count = 0
        self.visited = 0
        self.workflows = 0
        self.authenticated = False

    async def guard_requests(self, route: Route) -> None:
        request = route.request
        parsed = urlsplit(request.url)
        if parsed.netloc != self.origin and parsed.path.startswith("/_api/"):
            self.network_errors.append(f"Refused API request outside the verified demo: {request.url}")
            await route.abort("blockedbyclient")
            return
        if request.method not in ("GET", "HEAD", "OPTIONS"):
            if parsed.netloc != self.origin:
                self.blocked_writes.append(f"Cross-origin {request.method} {request.url}")
                await route.abort("blockedbyclient")
                return
            if parsed.path == "/_api/tournaments/demo-open/finalize-match" and self.mock_gate is not None:
                self.mock_count += 1
                await self.mock_gate.wait()
                await route.fulfill(status=503, json={"error": MOCK_ERROR})
                return
            if parsed.path not in ("/_api/login", "/_api/render-markdown"):
                self.blocked_writes.append(f"Unexpected {request.method} {parsed.path}")
                await route.fulfill(status=409, json={"error": "Smoke check blocked a data-changing request."})
                return
        await route.continue_()

    def observe(self, page: Page) -> None:
        page.on("pageerror", lambda error: self.page_errors.append(str(error)))

        def console_error(message) -> None:
            # HTTP errors are checked below, including expected anonymous 401s
            # and the deliberately mocked save failure.
            if message.type == "error" and not message.text.startswith("Failed to load resource:"):
                self.page_errors.append(message.text)

        page.on("console", console_error)

        def response_error(response) -> None:
            path = urlsplit(response.url).path
            if response.status < 400:
                return
            # Anonymous views probe optional personal registration and editor warnings.
            if response.status == 401 and (path == "/_api/me" or not self.authenticated):
                return
            if path == "/_api/tournaments/demo-open/finalize-match" and self.mock_gate is not None:
                return
            self.network_errors.append(f"HTTP {response.status} {response.url}")

        page.on("response", response_error)

    async def screenshot(self, page: Page, name: str, width: int) -> None:
        if self.screenshots:
            self.screenshots.mkdir(parents=True, exist_ok=True)
            await page.screenshot(
                path=str(self.screenshots / f"{width:04d}-{name}.png"), full_page=True, animations="disabled"
            )
            if name == "home" and width in (360, 1280):
                await page.screenshot(
                    path=str(self.screenshots / f"{width:04d}-home-viewport.png"), animations="disabled"
                )

    async def check_layout(self, page: Page, name: str, width: int) -> None:
        await page.evaluate("document.fonts.ready")
        await page.evaluate("() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))")
        layout = await page.evaluate(LAYOUT_DIAGNOSTICS)
        if layout["documentWidth"] > layout["width"] + 1:
            raise AssertionError(f"document horizontal overflow: {json.dumps(layout)}")
        if name == "run-match" and width < 576:
            await expect(page.locator("#points-table .winner-option").first).to_be_visible()
            points = await page.locator("#points-table").evaluate("""table => ({
                width: table.clientWidth, contentWidth: table.scrollWidth,
                winners: [...table.querySelectorAll('.winner-options-group')].map(el => el.querySelectorAll('.winner-option').length),
                inaccessible: [...table.querySelectorAll('.winner-option, .delete-point-btn, .set-step-btn')].flatMap(el => {
                    const r = el.getBoundingClientRect();
                    return r.width < 43.9 || r.height < 43.9 || r.left < -1 || r.right > innerWidth + 1
                        ? [{control: el.className, width: r.width, height: r.height, left: r.left, right: r.right}] : [];
                })
            })""")
            assert points["contentWidth"] <= points["width"] + 1, (
                f"Point controls require horizontal scrolling: {points}"
            )
            assert points["winners"] and all(count == 2 for count in points["winners"]), (
                f"Both teams must remain available on each point: {points}"
            )
            assert not points["inaccessible"], f"Phone point controls are clipped or smaller than 44px: {points}"
        assets = await page.evaluate("""() => ({
            sheets: [...document.querySelectorAll('link[rel=stylesheet]')].map(el => ({href: el.href, loaded: !!el.sheet})),
            fonts: [...document.fonts].filter(font => /Awesome/.test(font.family)).map(font => ({family: font.family, status: font.status})),
            theme: getComputedStyle(document.querySelector('.arctos-app') || document.body).getPropertyValue('--field-green').trim(),
            external: performance.getEntriesByType('resource').filter(entry => {
                const u = new URL(entry.name);
                return u.origin !== location.origin && /\\.(css|woff2?|ttf)(\\?|$)/.test(u.pathname);
            }).map(entry => entry.name)
        })""")
        assert assets["sheets"], "No stylesheet links found"
        for sheet in assets["sheets"]:
            assert urlsplit(sheet["href"]).netloc == self.origin, f"External stylesheet: {sheet['href']}"
            assert sheet["loaded"], f"Stylesheet did not load: {sheet['href']}"
        assert not assets["external"], f"External font/style dependencies: {assets['external']}"
        assert assets["theme"], "Arctos theme tokens are missing"
        invalid_patterns = await page.evaluate("""() => [...document.querySelectorAll('input[pattern]')].flatMap(el => {
            try { new RegExp(el.pattern, 'v'); return []; }
            catch (error) { return [{id: el.id, pattern: el.pattern, error: String(error)}]; }
        })""")
        assert not invalid_patterns, f"Invalid HTML validation patterns: {invalid_patterns}"
        # Every ordinary shell displays a Font Awesome brand/navigation icon.
        if await page.locator(".app-header").count():
            assert any(font["status"] == "loaded" for font in assets["fonts"]), "Local icon font did not load"
        assert not self.page_errors, f"JavaScript errors: {self.page_errors}"
        assert not self.network_errors, f"Failed HTTP resources: {self.network_errors}"
        assert not self.blocked_writes, f"Data-changing requests prevented: {self.blocked_writes}"
        await self.screenshot(page, name, width)

    async def visit(self, page: Page, screen: Screen, width: int) -> None:
        self.page_errors.clear()
        self.network_errors.clear()
        try:
            response = await page.goto(self.base_url + screen.path, wait_until="domcontentloaded")
            assert response and response.ok, f"Page returned HTTP {response.status if response else 'no response'}"
            await page.locator(screen.ready).first.wait_for(state="visible")
            # Avoid arbitrary sleeps: wait for the public data or form shell and
            # for any visible loading placeholder to leave the content region.
            await page.wait_for_function("""() => {
                const root = document.querySelector('main') || document.querySelector('.sc-enter');
                if (!root) return false;
                return ![...root.querySelectorAll('p, div, span')].some(el =>
                    el.children.length === 0 && el.getBoundingClientRect().height > 0 &&
                    /^Loading(?:[ .…]|$)/i.test(el.textContent.trim()));
            }""")
            await self.check_layout(page, screen.name, width)
            self.visited += 1
            print(f"PASS {width:4d}px {screen.path}", flush=True)
        except Exception as error:
            diagnostics = f"JS={self.page_errors}; HTTP={self.network_errors}; blocked writes={self.blocked_writes}"
            self.failures.append(f"{width}px {screen.path}: {error}\n  {diagnostics}")
            print(f"FAIL {self.failures[-1]}", flush=True)
            await self.screenshot(page, f"FAIL-{screen.name}", width)

    async def workflow(self, page: Page, name: str, width: int, action) -> None:
        self.page_errors.clear()
        self.network_errors.clear()
        try:
            await action()
            await self.check_layout(page, name, width)
            self.workflows += 1
            print(f"PASS {width:4d}px workflow: {name}", flush=True)
        except Exception as error:
            diagnostics = f"JS={self.page_errors}; HTTP={self.network_errors}; blocked writes={self.blocked_writes}"
            self.failures.append(f"{width}px workflow {name}: {error}\n  {diagnostics}")
            print(f"FAIL {self.failures[-1]}", flush=True)
            await self.screenshot(page, f"FAIL-{name}", width)
        finally:
            if self.mock_gate is not None:
                self.mock_gate.set()
                self.mock_gate = None

    async def discover_events(self, page: Page) -> None:
        await page.goto(self.base_url, wait_until="domcontentloaded")
        await expect(page.locator('.arctos-event-card a[href="/demo-summer-cup"]').first).to_be_visible()
        await expect(page.locator('.arctos-event-card a[href="/demo-spring-cup"]')).to_have_count(0)
        await page.get_by_role("button", name=re.compile(r"^Past\b")).click()
        await expect(page.locator('.arctos-event-card a[href="/demo-spring-cup"]').first).to_be_visible()
        await page.locator("#event-search").fill("no-such-demo-event")
        await expect(page.get_by_role("heading", name="No matching events")).to_be_visible()
        await page.get_by_role("button", name="Clear search", exact=True).click()
        await expect(page.locator("#event-search")).to_have_value("")
        await page.get_by_role("button", name=re.compile(r"^Upcoming\b")).click()
        await page.locator("#event-search").fill("Summer Cup")
        await expect(page.locator(".arctos-event-card")).to_have_count(1)
        await expect(page.locator('.arctos-event-card a[href="/demo-summer-cup"]').first).to_be_visible()

    async def registration_validation(self, page: Page) -> None:
        await page.goto(self.base_url + "/register/player", wait_until="domcontentloaded")
        await page.locator("#username").fill("demo-validation-only")
        await page.locator("#name").fill("Retain this display name")
        await page.locator("#password").fill("demo-first-password")
        await page.locator("#confirm_password").fill("demo-other-password")
        await page.get_by_role("button", name="Create account", exact=True).click()
        await expect(page.get_by_role("alert")).to_contain_text("Passwords do not match")
        await expect(page.locator("#username")).to_have_value("demo-validation-only")
        await expect(page.locator("#name")).to_have_value("Retain this display name")
        await expect(page.locator("#password")).to_have_value("demo-first-password")
        await expect(page.get_by_role("button", name="Create account", exact=True)).to_be_enabled()

    async def login(self, page: Page, username: str = DEMO_USER) -> None:
        await page.goto(self.base_url + "/login", wait_until="domcontentloaded")
        await page.get_by_label("Username", exact=True).fill(username)
        await page.get_by_label("Password", exact=True).fill(DEMO_PASSWORD)
        await page.get_by_role("button", name="Sign in", exact=True).click()
        await expect(page.locator("#event-search")).to_be_visible()
        result = await page.context.request.get(self.base_url + "/_api/me")
        assert result.ok and (await result.json())["id"] == username, f"Demo login failed: {username}"
        self.authenticated = True

    async def bracket_controls(self, page: Page) -> None:
        await page.goto(self.base_url + "/demo-open/bracket", wait_until="domcontentloaded")
        canvas = page.locator("#bracket-canvas-wrap")
        await expect(canvas).to_be_visible()
        assert await canvas.evaluate("el => getComputedStyle(el).touchAction") == "auto", (
            "Read-only bracket prevents normal phone scrolling"
        )
        await page.locator("#bracketEditMode").check()
        controls = page.get_by_role("group", name="Bracket canvas view", exact=True)
        await expect(controls).to_be_visible()
        await controls.get_by_role("button", name="Zoom in", exact=True).click()
        await expect(controls.locator("output")).to_have_text("120%")
        await controls.get_by_role("button", name="Move diagram right", exact=True).click()
        await controls.get_by_role("button", name="Reset view", exact=True).click()
        await expect(controls.locator("output")).to_have_text("100%")
        await page.locator("#bracketEditMode").uncheck()
        assert await canvas.evaluate("el => getComputedStyle(el).touchAction") == "auto"

    async def bracket_touch(self, page: Page) -> None:
        # Supply one movable node only in this browser response. Cancel the drag
        # instead of saving it, so the real demo bracket remains untouched.
        pattern = "**/_api/tournaments/demo-open/bracket"

        async def fixture(route: Route) -> None:
            if route.request.method != "GET":
                await self.guard_requests(route)
                return
            response = await route.fetch()
            data = await response.json()
            data["texts"] = [{"id": "touch-probe", "text": "Touch drag probe", "x_pos": 50, "y_pos": 80, "size": 18}]
            data["placements"] = []
            data["images"] = []
            data["labeled_teams"] = []
            await route.fulfill(response=response, json=data)

        await page.context.route(pattern, fixture)
        cdp = await page.context.new_cdp_session(page)
        touch_active = False
        try:
            await page.goto(self.base_url + "/demo-open/bracket", wait_until="domcontentloaded")
            await page.locator("#bracketEditMode").check()
            wrap = page.locator("#bracket-canvas-wrap")
            await wrap.scroll_into_view_if_needed()
            node = page.locator(".bracket-text")
            await expect(node).to_be_visible()
            box = await node.bounding_box()
            bounds = await wrap.bounding_box()
            assert box and bounds, "Touch bracket fixture is missing its node or canvas"
            original = await node.get_attribute("style")
            start = {"x": box["x"] + 40, "y": box["y"] + 10, "id": 1}
            await cdp.send("Input.dispatchTouchEvent", {"type": "touchStart", "touchPoints": [start]})
            touch_active = True
            await cdp.send(
                "Input.dispatchTouchEvent",
                {"type": "touchMove", "touchPoints": [{**start, "x": start["x"] + 40, "y": start["y"] + 35}]},
            )
            await expect(node).not_to_have_attribute("style", original)
            moved = await node.get_attribute("style")
            await cdp.send(
                "Input.dispatchTouchEvent",
                {"type": "touchMove", "touchPoints": [{**start, "x": start["x"] + 60, "y": max(20, bounds["y"] - 30)}]},
            )
            await expect(node).not_to_have_attribute("style", moved)
            await cdp.send("Input.dispatchTouchEvent", {"type": "touchCancel", "touchPoints": []})
            touch_active = False
            await expect(node).to_have_attribute("style", original)
            assert not self.blocked_writes, f"Touch cancellation tried to save: {self.blocked_writes}"

            canvas = page.locator("#bracket-canvas")
            transform = await canvas.get_attribute("style")
            viewport = page.viewport_size
            assert viewport
            blank = {
                "x": bounds["x"] + bounds["width"] - 45,
                "y": min(bounds["y"] + 280, viewport["height"] - 60),
                "id": 2,
            }
            await cdp.send("Input.dispatchTouchEvent", {"type": "touchStart", "touchPoints": [blank]})
            touch_active = True
            await cdp.send(
                "Input.dispatchTouchEvent",
                {"type": "touchMove", "touchPoints": [{**blank, "x": blank["x"] - 45, "y": blank["y"] - 55}]},
            )
            await cdp.send("Input.dispatchTouchEvent", {"type": "touchEnd", "touchPoints": []})
            touch_active = False
            await expect(canvas).not_to_have_attribute("style", transform)
            await node.dblclick()
            await expect(page.get_by_text("Edit Text", exact=True)).to_be_visible()
        finally:
            if touch_active:
                await cdp.send("Input.dispatchTouchEvent", {"type": "touchCancel", "touchPoints": []})
            await cdp.detach()
            await page.context.unroute(pattern, fixture)

    async def finalize_failure(self, page: Page, match_id: str) -> None:
        await page.goto(self.base_url + f"/demo-open/finalize-match/{match_id}", wait_until="domcontentloaded")
        await page.locator("#winner-team1").check()
        await page.locator("#final_notes").fill("Keep this note after a failed save.")
        for canvas_id in ("team1_signature_canvas", "team2_signature_canvas"):
            canvas = page.locator(f"#{canvas_id}")
            await canvas.scroll_into_view_if_needed()
            box = await canvas.bounding_box()
            assert box and box["width"] > 30, f"Signature canvas has no usable width: {canvas_id}"
            await page.mouse.move(box["x"] + 12, box["y"] + 25)
            await page.mouse.down()
            await page.mouse.move(box["x"] + min(90, box["width"] - 12), box["y"] + 60, steps=5)
            await page.mouse.up()
            await expect(canvas).to_have_attribute("data-signed", "true")
        self.mock_count = 0
        self.mock_gate = asyncio.Event()
        button = page.get_by_role("button", name="Confirm and finalize match", exact=True)
        await button.click()
        await expect(page.get_by_role("button", name="Finalizing match…", exact=True)).to_be_disabled()
        # A second form submission while pending must not send another request.
        await page.locator("form").dispatch_event("submit")
        self.mock_gate.set()
        await expect(page.get_by_role("alert")).to_contain_text(MOCK_ERROR)
        await expect(button).to_be_enabled()
        assert self.mock_count == 1, f"Expected one save request; got {self.mock_count}"
        await expect(page.locator("#final_notes")).to_have_value("Keep this note after a failed save.")
        await expect(page.locator("#winner-team1")).to_be_checked()
        for canvas_id in ("team1_signature_canvas", "team2_signature_canvas"):
            await expect(page.locator(f"#{canvas_id}")).to_have_attribute("data-signed", "true")


async def api_json(context: BrowserContext, base_url: str, path: str):
    response = await context.request.get(base_url + path)
    if not response.ok:
        raise RuntimeError(f"Demo discovery {path} returned HTTP {response.status}: {(await response.text())[:250]}")
    return await response.json()


def arguments(description: str | None = __doc__) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=description, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument(
        "--base-url", default="http://127.0.0.1:8080", help="Loopback Dioxus/proxy origin serving the demo"
    )
    parser.add_argument(
        "--browser-executable", type=Path, help="Installed Chromium executable, e.g. /usr/bin/google-chrome"
    )
    parser.add_argument("--screenshots", type=Path, help="Optional output directory for deterministic full-page PNGs")
    args = parser.parse_args()
    parsed = urlsplit(args.base_url)
    if (
        parsed.scheme not in ("http", "https")
        or parsed.hostname not in ("localhost", "127.0.0.1", "::1")
        or parsed.username
        or parsed.password
        or parsed.path not in ("", "/")
        or parsed.query
        or parsed.fragment
    ):
        parser.error("--base-url must be a loopback http(s) origin, without credentials, path, query or fragment")
    args.base_url = args.base_url.rstrip("/")
    if args.browser_executable and not args.browser_executable.is_file():
        parser.error(f"Browser executable does not exist: {args.browser_executable}")
    return args


async def run(args: argparse.Namespace) -> int:
    check = SmokeCheck(args.base_url, args.screenshots)
    async with async_playwright() as playwright:
        preflight = await playwright.request.new_context()
        try:
            response = await preflight.get(args.base_url + "/_api/demo-info", timeout=TIMEOUT_MS)
            try:
                marker = await response.json() if response.ok else {}
            except ValueError:
                marker = {}
            if marker.get("demo") is not True or marker.get("organizer") != DEMO_USER:
                raise RuntimeError("Refusing to run: backend is missing the scripts.dev_demo demo-only marker.")
        finally:
            await preflight.dispose()
        launch = {"headless": True}
        if args.browser_executable:
            launch["executable_path"] = str(args.browser_executable)
        browser = await playwright.chromium.launch(**launch)
        try:
            for width in WIDTHS:
                check.authenticated = False
                context = await browser.new_context(
                    viewport={"width": width, "height": 900}, has_touch=width < 768, is_mobile=width < 768
                )
                context.set_default_timeout(TIMEOUT_MS)
                await context.route("**/*", check.guard_requests)
                page = await context.new_page()
                check.observe(page)
                try:
                    for screen in PUBLIC_SCREENS:
                        await check.visit(page, screen, width)
                    await check.workflow(page, "event-search-and-dates", width, lambda: check.discover_events(page))
                    await check.workflow(
                        page, "registration-validation", width, lambda: check.registration_validation(page)
                    )
                    await check.workflow(page, "organizer-login", width, lambda: check.login(page))
                    # Stop this width if authentication failed; avoid misleading permission failures.
                    me = await api_json(context, args.base_url, "/_api/me")
                    assert me["id"] == DEMO_USER
                    schedule = await api_json(context, args.base_url, "/_api/tournaments/demo-open/schedule-setup")
                    matches = {match["name"]: match["uuid"] for match in schedule["matches"]}
                    if not {"Field A live match", "Next on Field B"} <= matches.keys():
                        raise RuntimeError("Demo matches missing; use an unmodified demo fixture for this check.")
                    live_id = matches["Field A live match"]
                    next_id = matches["Next on Field B"]
                    sidecomps = await api_json(context, args.base_url, "/_api/demo-open/sidecomps")
                    competition = next(comp for comp in sidecomps if comp["name"] == "Sideline Dueling — Demo")
                    comp_id = competition["id"]
                    dynamic = (
                        Screen("match", f"/demo-open/match/{live_id}", ".match-overview"),
                        Screen("start-match", f"/demo-open/start-match/{next_id}", "#match_notes"),
                        Screen("run-match", f"/demo-open/run-match/{live_id}", "#score-stones-card"),
                        Screen("finalize-match", f"/demo-open/finalize-match/{live_id}", "#final_notes"),
                        Screen("sidecomp", f"/demo-open/sidecomps/{comp_id}"),
                        Screen("sidecomp-edit", f"/demo-open/sidecomps/{comp_id}/edit"),
                        Screen(
                            "sidecomp-quick-register",
                            f"/demo-open/sidecomps/{comp_id}/register-player-as-to",
                            "main .list-group",
                        ),
                        Screen("sidecomp-results", f"/demo-open/sidecomps/{comp_id}/results"),
                        Screen("sidecomp-manage", f"/demo-open/sidecomps/{comp_id}/manage-results"),
                        Screen("sidecomp-scoring", f"/demo-open/sidecomps/{comp_id}/enter-results", ".sc-numpad"),
                    )
                    for screen in (*AUTH_SCREENS, *dynamic):
                        await check.visit(page, screen, width)
                    await check.workflow(page, "bracket-view-controls", width, lambda: check.bracket_controls(page))
                    if width < 768:
                        await check.workflow(page, "bracket-touch-and-cancel", width, lambda: check.bracket_touch(page))
                    await check.workflow(
                        page, "finalize-save-failure", width, lambda: check.finalize_failure(page, live_id)
                    )
                    await check.workflow(page, "team-login", width, lambda: check.login(page, "demo-team"))
                    for screen in (
                        Screen("event-invitations", "/demo-open/invitations"),
                        Screen("league-invitations", "/leagues/demo-coast-league/invitations"),
                        Screen("team-profile-edit", "/teams/demo-team/edit", "main form"),
                    ):
                        await check.visit(page, screen, width)
                finally:
                    if check.mock_gate:
                        check.mock_gate.set()
                    await context.close()
        finally:
            await browser.close()
    print(
        f"\n{check.visited} screen checks and {check.workflows} workflows passed; "
        f"{len(check.failures)} failures across widths {WIDTHS}."
    )
    if check.failures:
        print("\n".join(f"- {failure}" for failure in check.failures), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(asyncio.run(run(arguments())))
    except KeyboardInterrupt:
        raise SystemExit(130) from None
    except Exception as error:
        print(f"Smoke check stopped: {error}", file=sys.stderr)
        raise SystemExit(1) from error
