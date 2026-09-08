// Honk300's explicitly loaded KWin companion. No shortcuts or startup hooks.
// The native probe exercises this same script before Rust enables its capabilities.
// A script-global root retains the QObject throughout long-lived observations.
var honk300CompanionTimer = (function () {
    "use strict";
    var service = "org.emmetts.Honk300.Wayland";
    var path = "/org/emmetts/Honk300/KWin";
    var iface = "org.emmetts.Honk300.KWin1";
    var timer = new QTimer();
    timer.interval = 50;
    var sequence = 0;
    var pending = false;
    var pendingSince = 0;
    var bridgeOwner = null;
    var stopped = false;
    var lastResult = "none";
    function stop(reason) {
        stopped = true;
        pending = false;
        timer.stop();
        print("Honk300 KWin connection stopped: " + reason + "; sequence=" + sequence);
    }
    var protectedTokens = /^(terminal|console|xterm|uxterm|rxvt|urxvt|alacritty|kitty|foot|ghostty|wezterm|konsole|kgx|tilix|terminator|lxterminal|qterminal|blackbox|ptyxis|rio|code|codex|chatgpt|contour|tabby|warp|zellij|st|terminology|guake|yakuake|tilda|extraterm)$/;

    function boundedText(value) {
        var text = String(value === undefined || value === null ? "" : value);
        return text.length <= 256 ? text : "";
    }
    function windowList() {
        // Plasma 5 exposes the managed client list; Plasma 6 exposes stackingOrder.
        // The older list conveys identity, not top-to-bottom occlusion authority.
        if (workspace.stackingOrder !== undefined) return workspace.stackingOrder;
        if (typeof workspace.clientList === "function") return workspace.clientList();
        return null;
    }
    function protectedTarget(window) {
        var identity = [window.resourceClass, window.resourceName, window.desktopFileName, window.caption];
        if (!boundedText(window.resourceClass) || !boundedText(window.caption)) return true;
        for (var i = 0; i < identity.length; i++) {
            var value = boundedText(identity[i]);
            if (String(identity[i] || "").length > 256) return true;
            var tokens = value.toLowerCase().split(/[.\-_ :;,]+/);
            for (var j = 0; j < tokens.length; j++) {
                if (protectedTokens.test(tokens[j].replace(/[^a-z0-9]/g, ""))) return true;
            }
        }
        return false;
    }
    function rect(value) {
        return [Number(value.x), Number(value.y), Number(value.width), Number(value.height)];
    }
    function finite(value) {
        return typeof value === "number" && isFinite(value) && Math.abs(value) <= 1000000;
    }
    function goodRect(value) {
        return value.length === 4 && value.every(finite) && value[2] > 0 && value[3] > 0;
    }
    function onDesktop(window) {
        // Plasma 5's QtScript does not reliably expose QVector<VirtualDesktop*>.
        // Its numeric desktop property remains authoritative for this membership.
        if (typeof workspace.currentDesktop === "number") {
            return window.onAllDesktops === true || (typeof window.desktop === "number" &&
                window.desktop > 0 && window.desktop === workspace.currentDesktop);
        }
        var current = workspace.currentVirtualDesktop || workspace.currentDesktop;
        var desktops = window.desktops;
        if (!current || !desktops || typeof desktops.length !== "number") return false;
        if (desktops.length === 0) return true;
        for (var i = 0; i < desktops.length; i++) {
            if (desktops[i] === current || (boundedText(current.id) &&
                boundedText(desktops[i].id) === boundedText(current.id))) return true;
        }
        return false;
    }
    function onActivity(window) {
        var activities = window.activities;
        if (!activities || typeof activities.length !== "number") return false;
        if (activities.length === 0) return true;
        var current = boundedText(workspace.currentActivity);
        if (!current) return false;
        for (var i = 0; i < activities.length; i++) if (activities[i] === current) return true;
        return false;
    }
    function snapshot(window) {
        return {
            id: boundedText(window.internalId),
            pid: Number(window.pid),
            app: boundedText(window.resourceClass),
            title: boundedText(window.caption),
            geometry: rect(window.frameGeometry),
            area: rect(workspace.clientArea(KWin.ScreenArea, window)),
            normal: Boolean(window.normalWindow),
            deleted: Boolean(window.deleted),
            minimized: Boolean(window.minimized),
            fullscreen: Boolean(window.fullScreen),
            active: Boolean(window.active),
            moveable: Boolean(window.moveable),
            dragging: window.move === true,
            drag_known: typeof window.move === "boolean",
            on_desktop: onDesktop(window),
            on_activity: onActivity(window),
            protected: protectedTarget(window)
        };
    }
    function apply(command, frame) {
        if (!command || command.kind !== "move" || !Array.isArray(command.from) ||
            !Array.isArray(command.to) || command.to.length !== 2 || !command.to.every(finite)) return "invalid";
        var windows = windowList();
        if (!windows || windows.length > 64) return "unavailable";
        for (var i = 0; i < windows.length; i++) {
            var window = windows[i];
            if (boundedText(window.internalId) !== command.id) continue;
            var current = snapshot(window);
            if (current.protected) return "protected";
            if (!current.normal || current.deleted || current.minimized || current.fullscreen ||
                !current.moveable || current.dragging || !current.on_desktop || !current.on_activity) return "ineligible";
            if (current.pid !== command.pid || current.app !== command.app ||
                JSON.stringify(current.geometry) !== JSON.stringify(command.from)) return "stale";
            var observed = frame.windows.filter(function (item) { return item.id === current.id; });
            if (observed.length !== 1 || JSON.stringify(observed[0]) !== JSON.stringify(current)) return "changed";
            if (!goodRect(current.geometry) || !goodRect(current.area)) return "invalid";
            var dx = command.to[0] - current.geometry[0];
            var dy = command.to[1] - current.geometry[1];
            if (dx * dx + dy * dy > 24 * 24) return "bounded";
            var area = current.area;
            if (command.to[0] < area[0] || command.to[1] < area[1] ||
                command.to[0] + current.geometry[2] > area[0] + area[2] ||
                command.to[1] + current.geometry[3] > area[1] + area[3]) return "outside";
            var geometry = window.frameGeometry;
            geometry.x = command.to[0];
            geometry.y = command.to[1];
            window.frameGeometry = geometry;
            return "moved";
        }
        return "gone";
    }
    function exchange() {
        if (stopped) return;
        if (pending) {
            if (Date.now() - pendingSince >= 250 || Date.now() < pendingSince) {
                stop("exchange deadline");
            }
            return;
        }
        var windows = windowList();
        if (!windows || windows.length > 64) { stop("window inventory unavailable or oversized"); return; }
        var frame = {protocol: 1, sequence: ++sequence, windows: [],
            stacking_order: workspace.stackingOrder !== undefined,
            pointer: [Number(workspace.cursorPos.x), Number(workspace.cursorPos.y)], result: lastResult};
        for (var i = 0; i < windows.length; i++) frame.windows.push(snapshot(windows[i]));
        var bytes = JSON.stringify(frame);
        if (bytes.length > 65536) { stop("snapshot too large"); return; }
        pending = true;
        var started = Date.now();
        pendingSince = started;
        callDBus("org.freedesktop.DBus", "/org/freedesktop/DBus", "org.freedesktop.DBus", "GetNameOwner", service, function (owner) {
            if (stopped) return;
            if (typeof owner !== "string" || owner.charAt(0) !== ":" ||
                (bridgeOwner !== null && bridgeOwner !== owner) ||
                Date.now() - started >= 250 || Date.now() < started) {
                stop("bridge owner missing, replaced or late"); return;
            }
            bridgeOwner = owner;
            // Address the pinned unique owner, never a replacement claiming its name.
            callDBus(bridgeOwner, path, iface, "Exchange", bytes, function (reply) {
            pending = false;
            if (stopped) return;
            if (Date.now() - started >= 250 || Date.now() < started || typeof reply !== "string" || reply.length > 4096) {
                stop("bridge reply invalid or late"); return;
            }
            try {
                var message = JSON.parse(reply);
                if (message.protocol !== 1 || message.sequence !== frame.sequence ||
                    !Array.isArray(message.commands) || message.commands.length > 1) { stop("invalid response schema"); return; }
                if (message.stop === true) { stop("explicit revoke"); return; }
                lastResult = message.commands.length ? apply(message.commands[0], frame) : "none";
            } catch (error) { stop("response decoding failed"); }
            });
        });
    }
    timer.timeout.connect(exchange);
    timer.start();
    return timer;
}());
