// Exercise the real macOS Accessibility provider against an exact built companion.
// Compile with: xcrun clang -fobjc-arc -framework Cocoa -framework ApplicationServices ...
#import <Cocoa/Cocoa.h>
#import <ApplicationServices/ApplicationServices.h>
#import <signal.h>

static NSTask *settings;
static AXUIElementRef application;
static NSString *evidenceDirectory;

static void require(BOOL condition, NSString *message) {
    if (!condition) @throw [NSException exceptionWithName:@"HonkSettingsAXFailure" reason:message userInfo:nil];
}

static id attribute(id element, CFStringRef name) {
    if (!element) return nil;
    CFTypeRef value = NULL;
    AXError error = AXUIElementCopyAttributeValue((__bridge AXUIElementRef)element, name, &value);
    if (error != kAXErrorSuccess) {
        if (value) CFRelease(value);
        return nil;
    }
    return CFBridgingRelease(value);
}

static NSArray *tree(void) {
    NSMutableArray *pending = [NSMutableArray arrayWithObject:(__bridge id)application];
    NSMutableArray *nodes = [NSMutableArray array];
    while (pending.count && nodes.count < 640) {
        id node = pending.lastObject;
        [pending removeLastObject];
        [nodes addObject:node];
        id children = attribute(node, kAXChildrenAttribute);
        if ([children isKindOfClass:[NSArray class]]) [pending addObjectsFromArray:children];
    }
    require(!pending.count, @"Native accessibility tree exceeded the bounded fixture traversal");
    return nodes;
}

static NSString *stringAttribute(id node, CFStringRef name) {
    id value = attribute(node, name);
    return [value isKindOfClass:[NSString class]] ? value : @"";
}

static NSString *nodeName(id node) {
    NSString *title = stringAttribute(node, kAXTitleAttribute);
    if (title.length) return title;
    NSString *description = stringAttribute(node, kAXDescriptionAttribute);
    if (description.length) return description;
    return stringAttribute(node, kAXValueAttribute);
}

static id find(NSString *name, NSString *role) {
    for (id node in tree()) {
        if ([nodeName(node) isEqualToString:name] && (!role || [stringAttribute(node, kAXRoleAttribute) isEqualToString:role])) return node;
    }
    return nil;
}

static void waitFor(BOOL (^check)(void), NSString *description) {
    double deadline = NSProcessInfo.processInfo.systemUptime + 20;
    while (NSProcessInfo.processInfo.systemUptime < deadline) {
        require(settings.running, [NSString stringWithFormat:@"Settings exited while waiting for %@", description]);
        if (check()) return;
        [NSThread sleepForTimeInterval:0.1];
    }
    require(NO, [NSString stringWithFormat:@"Native Accessibility did not provide %@", description]);
}

static id waitForNode(NSString *name, NSString *role) {
    __block id node = nil;
    waitFor(^BOOL { node = find(name, role); return node != nil; }, name);
    return node;
}

static void press(NSString *name, NSString *role) {
    id node = waitForNode(name, role);
    require([attribute(node, kAXEnabledAttribute) boolValue], [NSString stringWithFormat:@"%@ is not enabled", name]);
    AXError error = AXUIElementPerformAction((__bridge AXUIElementRef)node, kAXPressAction);
    require(error == kAXErrorSuccess, [NSString stringWithFormat:@"Native press failed for %@: %d", name, error]);
}

static BOOL checked(NSString *name) {
    id node = waitForNode(name, @"AXCheckBox");
    id value = attribute(node, kAXValueAttribute);
    require(value != nil && [value respondsToSelector:@selector(doubleValue)],
            [NSString stringWithFormat:@"%@ has no readable native checked state", name]);
    double number = [value doubleValue];
    require(number == 0 || number == 1, [NSString stringWithFormat:@"%@ has invalid native checked state %@", name, value]);
    return number == 1;
}

static void writeJSON(NSString *name, id value) {
    NSError *error = nil;
    NSData *data = [NSJSONSerialization dataWithJSONObject:value options:NSJSONWritingPrettyPrinted error:&error];
    require(data != nil, error.localizedDescription ?: @"Could not encode accessibility evidence");
    require([data writeToFile:[evidenceDirectory stringByAppendingPathComponent:name] options:NSDataWritingAtomic error:&error],
            error.localizedDescription ?: @"Could not save accessibility evidence");
}

static void captureTree(NSString *name) {
    NSMutableArray *records = [NSMutableArray array];
    for (id node in tree()) {
        NSMutableDictionary *record = [NSMutableDictionary dictionaryWithDictionary:@{
            @"name":nodeName(node), @"role":stringAttribute(node, kAXRoleAttribute),
            @"identifier":stringAttribute(node, kAXIdentifierAttribute)}];
        for (NSString *key in @[@"AXValue", @"AXEnabled", @"AXFocused"]) {
            id value = attribute(node, (__bridge CFStringRef)key);
            if ([value isKindOfClass:[NSString class]] || [value isKindOfClass:[NSNumber class]]) record[key] = value;
        }
        CFArrayRef actions = NULL;
        if (AXUIElementCopyActionNames((__bridge AXUIElementRef)node, &actions) == kAXErrorSuccess && actions) record[@"actions"] = CFBridgingRelease(actions);
        [records addObject:record];
    }
    writeJSON(name, records);
}

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc != 3 && (argc != 4 || strcmp(argv[3], "--independent-presence") != 0)) { fprintf(stderr, "usage: settings-ax BINARY FRESH_EVIDENCE_DIRECTORY [--independent-presence]\n"); return 2; }
        BOOL independentPresence = argc == 4;
        NSString *binary = [[NSString stringWithUTF8String:argv[1]] stringByStandardizingPath];
        evidenceDirectory = [[NSString stringWithUTF8String:argv[2]] stringByStandardizingPath];
        NSFileManager *files = NSFileManager.defaultManager;
        [files createDirectoryAtPath:evidenceDirectory withIntermediateDirectories:YES attributes:nil error:NULL];
        NSString *config = [evidenceDirectory stringByAppendingPathComponent:@"config.toml"];
        __block int result = 1;
        @try {
            require(AXIsProcessTrusted(), @"The disposable runner did not authorize native Accessibility queries; no permission prompt was requested");
            require(![files fileExistsAtPath:config], @"Native Accessibility fixture requires a fresh isolated config");
            require([@"# Native macOS Accessibility fixture\ngoose_config_version = 2\n" writeToFile:config atomically:YES encoding:NSUTF8StringEncoding error:NULL], @"Could not create isolated config");
            NSString *logPath = [evidenceDirectory stringByAppendingPathComponent:@"process.log"];
            [files createFileAtPath:logPath contents:nil attributes:nil];
            NSFileHandle *log = [NSFileHandle fileHandleForWritingAtPath:logPath];
            settings = [NSTask new];
            settings.executableURL = [NSURL fileURLWithPath:binary];
            settings.arguments = @[@"--config", config];
            settings.standardOutput = log;
            settings.standardError = log;
            settings.standardInput = [NSFileHandle fileHandleWithNullDevice];
            NSError *launchError = nil;
            require([settings launchAndReturnError:&launchError], launchError.localizedDescription ?: @"Settings launch failed");
            application = AXUIElementCreateApplication(settings.processIdentifier);
            AXUIElementSetMessagingTimeout(application, 0.25);
            waitForNode(@"First wander (seconds)", @"AXButton");
            captureTree(@"general-tree.json");
            press(@"Appearance", @"AXButton");
            require(!checked(@"Reduced motion"), @"Isolated reduced motion did not begin disabled");
            press(@"Reduced motion", @"AXCheckBox");
            waitFor(^BOOL { return checked(@"Reduced motion"); }, @"changed native checkbox state");
            press(@"Save & apply", @"AXButton");
            waitFor(^BOOL { return [[NSString stringWithContentsOfFile:config encoding:NSUTF8StringEncoding error:NULL] containsString:@"reduced_motion = true"]; }, @"persisted switch");
            press(@"General", @"AXButton");
            id backgroundReference = waitForNode(@"Appearance", @"AXButton");
            press(@"First wander (seconds)", @"AXButton");
            waitForNode(@"Edit setting", @"AXGroup");
            require(find(@"General", @"AXButton") == nil, @"Native modal still exposes background page actions");
            require(AXUIElementPerformAction((__bridge AXUIElementRef)backgroundReference, kAXPressAction) != kAXErrorSuccess,
                    @"A retained background reference still accepts an action during the modal");
            id field = waitForNode(@"First wander (seconds)", @"AXTextField");
            require([stringAttribute(field, kAXValueAttribute) isEqualToString:@"20"], @"Native reader cannot read initial editor text");
            id cancel = waitForNode(@"Cancel", @"AXButton");
            require(AXUIElementSetAttributeValue((__bridge AXUIElementRef)cancel, kAXFocusedAttribute, kCFBooleanTrue) == kAXErrorSuccess, @"Native reader could not move focus away from the editor");
            waitFor(^BOOL { return [attribute(find(@"Cancel", @"AXButton"), kAXFocusedAttribute) boolValue] && ![attribute(find(@"First wander (seconds)", @"AXTextField"), kAXFocusedAttribute) boolValue]; }, @"actual focus transfer away from the editor");
            field = waitForNode(@"First wander (seconds)", @"AXTextField");
            require(AXUIElementSetAttributeValue((__bridge AXUIElementRef)field, kAXFocusedAttribute, kCFBooleanTrue) == kAXErrorSuccess, @"Native reader could not focus the editor");
            waitFor(^BOOL { return [attribute(find(@"First wander (seconds)", @"AXTextField"), kAXFocusedAttribute) boolValue]; }, @"actual editor focus");
            field = waitForNode(@"First wander (seconds)", @"AXTextField");
            require(AXUIElementSetAttributeValue((__bridge AXUIElementRef)field, kAXValueAttribute, CFSTR("25")) == kAXErrorSuccess, @"Native reader could not edit text");
            waitFor(^BOOL { return [stringAttribute(find(@"First wander (seconds)", @"AXTextField"), kAXValueAttribute) isEqualToString:@"25"]; }, @"changed readable editor text");
            captureTree(@"editor-tree.json");
            press(@"Apply to draft", @"AXButton");
            waitForNode(@"General", @"AXButton");
            press(@"Save & apply", @"AXButton");
            waitFor(^BOOL { return [[NSString stringWithContentsOfFile:config encoding:NSUTF8StringEncoding error:NULL] containsString:@"first_wander_time_seconds = 25"]; }, @"saved editor value");
            waitForNode(@"Saved. The goose will use these settings when it starts.", @"AXStaticText");
            press(@"Behavior", @"AXButton");
            for (NSString *name in @[@"Honk on the hour", @"Travel across monitors", @"Allow cursor nabs", @"Random cursor nabs", @"Prevent all cursor nabs", @"Prevent window rides", @"Ride supported windows", @"Bring notes and memes"]) {
                BOOL previous = checked(name);
                press(name, @"AXCheckBox");
                waitFor(^BOOL { return checked(name) != previous; }, [@"changed " stringByAppendingString:name]);
            }
            press(@"Save & apply", @"AXButton");
            waitFor(^BOOL { NSString *saved = [NSString stringWithContentsOfFile:config encoding:NSUTF8StringEncoding error:NULL]; return [saved containsString:@"no_mouse_steal = true"] && [saved containsString:@"can_attack_mouse = false"]; }, @"saved complete dirty page");
            press(@"Platform & status", @"AXButton");
            if (independentPresence) waitFor(^BOOL { for (id node in tree()) { NSString *name = nodeName(node); if ([name containsString:@"Goose: stopped"] && [name containsString:@"Fullscreen observation: unprobed"] && [name containsString:@"Do not disturb: unprobed"]) return YES; } return NO; }, @"independent native presence status");
            id update = waitForNode(@"Update now", @"AXButton");
            require(attribute(update, kAXEnabledAttribute) != nil && ![attribute(update, kAXEnabledAttribute) boolValue], @"Unavailable update was exposed as enabled");
            CFArrayRef actions = NULL;
            AXError actionResult = AXUIElementCopyActionNames((__bridge AXUIElementRef)update, &actions);
            require(actionResult == kAXErrorSuccess, @"Native reader cannot query disabled update actions");
            NSArray *disabledActions = CFBridgingRelease(actions);
            require(![disabledActions containsObject:(__bridge NSString *)kAXPressAction], @"Disabled update still advertises a press action");
            captureTree(@"platform-tree.json");
            NSMutableArray *checks = [NSMutableArray arrayWithArray:@[@"native-names",@"action",@"checkbox-state",@"text-interface",@"focus",@"native-text-edit",@"modal-isolation",@"stale-modal-action",@"save-readback",@"largest-dirty-page",@"disabled-button-state"]];
            if (independentPresence) [checks addObject:@"independent-presence-status"];
            writeJSON(@"result.json", @{@"schema":@"honk300.settings-macos-ax-smoke.v1", @"ok":@YES, @"pid":@(settings.processIdentifier), @"automation_harness":@NO,
                @"independent_presence":@(independentPresence), @"checks":checks});
            result = 0;
        } @catch (NSException *exception) {
            fprintf(stderr, "%s\n", exception.reason.UTF8String);
            @try { if (application) captureTree(@"failed-tree.json"); writeJSON(@"failure.json", @{@"ok":@NO,@"reason":exception.reason ?: @"unknown"}); } @catch (NSException *ignored) { (void)ignored; }
        } @finally {
            if (settings.running) {
                [settings terminate];
                double deadline = NSProcessInfo.processInfo.systemUptime + 5;
                while (settings.running && NSProcessInfo.processInfo.systemUptime < deadline) [NSThread sleepForTimeInterval:0.05];
                if (settings.running) kill(settings.processIdentifier, SIGKILL);
                [settings waitUntilExit];
            }
            if (application) CFRelease(application);
        }
        return result;
    }
}
