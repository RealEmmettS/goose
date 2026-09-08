// Honk300's explicitly loaded KWin companion. No shortcuts or startup hooks.
// The native probe exercises this same script before Rust enables its capabilities.
(function () {
    "use strict";
    var service = "org.emmetts.Honk300.Wayland";
    var path = "/org/emmetts/Honk300/KWin";
    var iface = "org.emmetts.Honk300.KWin1";
    var timer = new QTimer();
    timer.interval = 50;
    var sequence = 0;
    var pending = false;
    var stopped = false;
    var lastResult = "none";
    var protectedTokens = /^(terminal|console|xterm|uxterm|rxvt|urxvt|alacritty|kitty|foot|ghostty|wezterm|konsole|kgx|tilix|terminator|lxterminal|qterminal|blackbox|ptyxis|rio|code|codex|chatgpt|contour|tabby|warp|zellij|st|terminology|guake|yakuake|tilda|extraterm)$/;

    function boundedText(value) {
        var text = String(value === undefined || value === null ? "" : value);
        return text.length <= 256 ? text : "";
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
            dragging: Boolean(window.interactiveMove || window.move),
            protected: protectedTarget(window)
        };
    }
    function apply(command, frame) {
        if (!command || command.kind !== "move" || !Array.isArray(command.from) ||
            !Array.isArray(command.to) || command.to.length !== 2 || !command.to.every(finite)) return "invalid";
        var windows = workspace.stackingOrder;
        if (!windows || windows.length > 64) return "unavailable";
        for (var i = 0; i < windows.length; i++) {
            var window = windows[i];
            if (boundedText(window.internalId) !== command.id) continue;
            var current = snapshot(window);
            if (current.protected) return "protected";
            if (!current.normal || current.deleted || current.minimized || current.fullscreen || !current.moveable) return "ineligible";
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
        if (stopped || pending) return;
        var windows = workspace.stackingOrder;
        if (!windows || windows.length > 64) { stopped = true; timer.stop(); return; }
        var frame = {protocol: 1, sequence: ++sequence, windows: [],
            pointer: [Number(workspace.cursorPos.x), Number(workspace.cursorPos.y)], result: lastResult};
        for (var i = 0; i < windows.length; i++) frame.windows.push(snapshot(windows[i]));
        var bytes = JSON.stringify(frame);
        if (bytes.length > 65536) { stopped = true; timer.stop(); return; }
        pending = true;
        var started = Date.now();
        callDBus(service, path, iface, "Exchange", bytes, function (reply) {
            pending = false;
            if (stopped || Date.now() - started > 250 || typeof reply !== "string" || reply.length > 4096) return;
            try {
                var message = JSON.parse(reply);
                if (message.protocol !== 1 || message.sequence !== frame.sequence ||
                    !Array.isArray(message.commands) || message.commands.length > 1) return;
                if (message.stop === true) { stopped = true; timer.stop(); return; }
                lastResult = message.commands.length ? apply(message.commands[0], frame) : "none";
            } catch (error) { stopped = true; timer.stop(); }
        });
    }
    timer.timeout.connect(exchange);
    timer.start();
}());
