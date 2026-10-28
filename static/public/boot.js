// Start the application once and hand navigation over to its router.
(() => {
  const boot = document.getElementById("app-boot");
  if (!boot || !boot.dataset.entry) return;
  const startup = document.getElementById("app-startup") || boot;
  const main = document.getElementById("main");
  if (!main) return;
  const loadingEntry = boot.dataset.loadingEntry === "true";
  const root = document.documentElement;
  const status = boot.querySelector("[data-boot-message]");
  const detail = boot.querySelector("[data-boot-detail]");
  const heading = boot.querySelector("[data-boot-heading]");
  const activity = boot.querySelector(".startup-activity");
  const retry = boot.querySelector("[data-boot-retry]");
  let finished = false;
  let failed = false;
  let pending = loadingEntry ? location.href : null;
  let selected = location.href;
  let waitingForRoute = false;
  let handoffFrame = null;

  function stopListening() {
    clearTimeout(slow);
    clearTimeout(recovery);
    cancelAnimationFrame(handoffFrame);
    observer.disconnect();
    window.removeEventListener("unhandledrejection", fail);
    window.removeEventListener("error", resourceError, true);
    window.removeEventListener("arctos:router-ready", navigate);
    document.removeEventListener("load", inspect, true);
    document.removeEventListener("click", click);
  }

  function fail() {
    if (finished || failed) return;
    failed = true;
    boot.dataset.bootState = "failed";
    status.textContent = `${boot.dataset.workflow} could not open. Please try again.`;
    if (detail) detail.textContent = "Retry to open your selected page.";
    if (activity) activity.hidden = true;
    retry.href = selected;
    retry.hidden = false;
    stopListening();
  }

  function ready() {
    if (finished || failed) return;
    finished = true;
    stopListening();
    boot.dataset.bootState = "ready";
    // Layout is ready to own its route loading/content/error states. Hide the
    // temporary chrome in the same task so there is never a visible second header.
    const restoreFocus = startup.contains(document.activeElement);
    boot.hidden = true;
    startup.hidden = true;
    main.removeAttribute("aria-hidden");
    main.removeAttribute("inert");
    document.body.classList.remove("public-startup");
    if (restoreFocus) {
      // Let the reveal paint before focusing the live route. A hidden startup
      // control can retain focus until that paint; preserve any application
      // control the visitor has selected in the meantime.
      requestAnimationFrame(() => requestAnimationFrame(() => {
        if (document.activeElement === document.body || startup.contains(document.activeElement)) {
          main.querySelector("#app-layout > #page-content")?.focus({preventScroll: true});
        }
      }));
    }
    window.dispatchEvent(new Event("arctos:application-visible"));
  }

  function inspect() {
    if (finished || failed || pending || waitingForRoute) return;
    // The route owns its data loading and error states after this handoff.
    const content = main.querySelector("#app-layout > #page-content");
    if (!content?.childElementCount) return;
    if (loadingEntry) {
      // Only the app requests Bootstrap in this mode. Its links live in #main;
      // wait for their styles, not optional font files, before revealing it.
      const styles = [...main.querySelectorAll('link[rel="stylesheet"]')];
      if (!styles.length || styles.some(link => !link.sheet)) return;
      if (handoffFrame !== null) return;
      handoffFrame = requestAnimationFrame(() => {
        handoffFrame = null;
        if (!pending && !waitingForRoute) ready();
      });
    } else ready();
  }

  function resourceError(event) {
    const target = event.target;
    const scriptFailed = target?.tagName === "SCRIPT" || event.filename?.includes("/assets/");
    const stylesheetFailed = loadingEntry && target?.tagName === "LINK"
      && target.rel === "stylesheet" && main.contains(target);
    if (scriptFailed || stylesheetFailed) fail();
  }

  function destination(value) {
    const url = new URL(value, location.href);
    if (url.origin !== location.origin || !["http:", "https:"].includes(url.protocol)) return null;
    const prefix = (root.dataset.publicPrefix || "").replace(/\/$/, "");
    if (prefix && url.pathname !== prefix && !url.pathname.startsWith(prefix + "/")) return null;
    const path = url.pathname.slice(prefix.length) || "/";
    if (/^\/(?:_api|api|static|assets|waivers)(?:\/|$)/.test(path)) return null;
    if (path === "/scoreboard-render" || /\/(?:scoreboard|enter-results)$/.test(path)) return null;
    if (url.pathname === location.pathname && url.search === location.search && url.hash) return null;
    return url.href;
  }

  function navigate() {
    if (!loadingEntry || finished || failed || !pending || root.dataset.arctosRouterReady !== "true") return;
    const target = pending;
    // An accepted URL is not yet a rendered route. A subsequent route mutation
    // must occur before handoff, including a second click while styles load.
    waitingForRoute = target !== location.href || !main.querySelector("#app-layout > #page-content")?.childElementCount;
    status.textContent = `Opening ${boot.dataset.workflow}…`;
    if (detail) detail.textContent = "Arctos has started. Your page is opening.";
    const event = new CustomEvent("arctos:navigate", {detail: target, cancelable: true});
    window.dispatchEvent(event);
    if (!event.defaultPrevented) { location.assign(target); return; }
    pending = null;
    // Re-entering the identical URL activates the parked Router without adding
    // a history entry. Changed destinations use the existing application history.
    inspect();
  }

  function select(target, source) {
    selected = pending = target;
    retry.href = target;
    const label = source?.querySelector("span")?.textContent || source?.textContent?.trim();
    if (label) {
      boot.dataset.workflow = source?.classList.contains("app-brand") ? "Events" : label;
      heading.textContent = boot.dataset.workflow;
    }
    const path = new URL(target).pathname;
    startup.querySelectorAll(".app-nav-link").forEach(link => {
      const active = new URL(link.href).pathname === path;
      link.classList.toggle("active", active);
      if (active) link.setAttribute("aria-current", "page");
      else link.removeAttribute("aria-current");
    });
    if (root.dataset.arctosRouterReady !== "true") {
      status.textContent = `Starting Arctos to open ${boot.dataset.workflow}…`;
    }
    navigate();
  }

  function click(event) {
    if (!loadingEntry || finished || failed) return;
    const link = event.target.closest?.("a[href]");
    if (!link || !startup.contains(link) || event.defaultPrevented
        || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey
        || link.hasAttribute("download") || link.hasAttribute("data-boot-retry")
        || link.hasAttribute("data-native-navigation") || (link.target && link.target !== "_self")) return;
    const target = destination(link.href);
    if (!target) return;
    event.preventDefault();
    select(target, link);
  }

  const slow = setTimeout(() => {
    if (finished || failed) return;
    boot.dataset.bootState = "slow";
    if (loadingEntry) {
      status.textContent = root.dataset.arctosRouterReady === "true" ? `Still opening ${boot.dataset.workflow}…` : "Still starting Arctos…";
      detail.textContent = "This is taking longer than usual. You can keep this page open while it finishes.";
    } else status.textContent = "Still loading. This is taking longer than usual. You can go back while it finishes.";
  }, 5000);
  const recovery = setTimeout(() => {
    if (finished || failed) return;
    retry.href = selected;
    retry.hidden = false;
    if (detail) detail.textContent = "You can keep waiting, or retry opening your selected page.";
  }, 20000);
  const observer = new MutationObserver(records => {
    if (waitingForRoute && records.some(record => record.type === "childList" && (
      record.target === main || record.target.closest?.("#app-layout > #page-content")
      || [...record.addedNodes].some(node => node.nodeType === 1 && (node.id === "app-layout" || node.querySelector?.("#app-layout")))
    ))) waitingForRoute = false;
    inspect();
  });
  observer.observe(main, {childList: true, subtree: true, attributes: true});
  document.addEventListener("load", inspect, true);
  document.addEventListener("click", click);
  window.addEventListener("arctos:router-ready", navigate);
  window.addEventListener("unhandledrejection", fail);
  window.addEventListener("error", resourceError, true);
  // The generated entry starts Wasm itself. import() completion is not runtime
  // readiness; only the bridge can activate the latest requested destination.
  import(boot.dataset.entry).catch(fail);
  navigate();
})();
