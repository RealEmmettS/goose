//! The same bounded, typed setting catalogue is consumed by the native window.
use honk_config::Config;
use serde::Serialize;
use serde_json::Value;

pub(super) struct FieldSpec {
    pub page: &'static str,
    pub key: &'static str,
    pub label: &'static str,
    pub help: &'static str,
}
pub(super) const FIELDS: &[FieldSpec] = &[
    FieldSpec {
        page: "General",
        key: "lifecycle.autostart_on_login",
        label: "Start at login",
        help: "Starts automatically when you sign in.",
    },
    FieldSpec {
        page: "General",
        key: "behavior.first_wander_time_seconds",
        label: "First wander (seconds)",
        help: "",
    },
    FieldSpec {
        page: "General",
        key: "behavior.min_wandering_time_seconds",
        label: "Shortest wander (seconds)",
        help: "",
    },
    FieldSpec {
        page: "General",
        key: "behavior.max_wandering_time_seconds",
        label: "Longest wander (seconds)",
        help: "",
    },
    FieldSpec {
        page: "Appearance",
        key: "appearance.expressions",
        label: "Expressive reactions",
        help: "Blinking, head movement, honks and pleased reactions.",
    },
    FieldSpec {
        page: "Appearance",
        key: "appearance.reduced_motion",
        label: "Reduced motion",
        help: "Keeps walking while reducing secondary motion.",
    },
    FieldSpec {
        page: "Appearance",
        key: "appearance.calm_goose",
        label: "Calm movements",
        help: "Reduces sudden turns and activity.",
    },
    FieldSpec {
        page: "Appearance",
        key: "behavior.use_custom_colors",
        label: "Custom colors",
        help: "Saved colors are retained while this is off.",
    },
    FieldSpec {
        page: "Appearance",
        key: "colors.goose_white",
        label: "Body color",
        help: "Hex color, for example #ffffff.",
    },
    FieldSpec {
        page: "Appearance",
        key: "colors.goose_orange",
        label: "Bill and feet color",
        help: "",
    },
    FieldSpec {
        page: "Appearance",
        key: "colors.goose_outline",
        label: "Outline color",
        help: "",
    },
    FieldSpec {
        page: "Appearance",
        key: "colors.goose_shade",
        label: "Body shading",
        help: "Blank derives this color from the main palette.",
    },
    FieldSpec {
        page: "Appearance",
        key: "colors.goose_wing",
        label: "Wing color",
        help: "Blank derives this color from the main palette.",
    },
    FieldSpec {
        page: "Appearance",
        key: "colors.goose_orange_dark",
        label: "Bill shading",
        help: "Blank derives this color from the main palette.",
    },
    FieldSpec {
        page: "Behavior",
        key: "behaviors.on_hour_double_honk",
        label: "Honk on the hour",
        help: "Respects quiet hours.",
    },
    FieldSpec {
        page: "Behavior",
        key: "behaviors.multi_monitor_chase",
        label: "Travel across monitors",
        help: "",
    },
    FieldSpec {
        page: "Behavior",
        key: "behavior.can_attack_mouse",
        label: "Allow cursor nabs",
        help: "Every nab remains bounded and respects permissions.",
    },
    FieldSpec {
        page: "Behavior",
        key: "behavior.attack_randomly",
        label: "Random cursor nabs",
        help: "Off by default.",
    },
    FieldSpec {
        page: "Behavior",
        key: "safety.no_mouse_steal",
        label: "Prevent all cursor nabs",
        help: "Takes priority over the other nab settings.",
    },
    FieldSpec {
        page: "Behavior",
        key: "safety.no_window_ride",
        label: "Prevent window rides",
        help: "Terminal windows are always protected.",
    },
    FieldSpec {
        page: "Behavior",
        key: "mischief.perch_and_ride",
        label: "Ride supported windows",
        help: "Requires a capable desktop session.",
    },
    FieldSpec {
        page: "Behavior",
        key: "mischief.collect_windows",
        label: "Bring notes and memes",
        help: "Requires a capable desktop session.",
    },
    FieldSpec {
        page: "Behavior",
        key: "mischief.collect_notes",
        label: "Bring notes",
        help: "",
    },
    FieldSpec {
        page: "Behavior",
        key: "mischief.collect_memes",
        label: "Bring memes",
        help: "",
    },
    FieldSpec {
        page: "Behavior",
        key: "interaction.pat_streak",
        label: "React to petting",
        help: "",
    },
    FieldSpec {
        page: "Behavior",
        key: "moods.dynamic_moods",
        label: "Changing moods",
        help: "",
    },
    FieldSpec {
        page: "Behavior",
        key: "moods.mood_intensity",
        label: "Mood intensity",
        help: "Choose calm, normal or spicy.",
    },
    FieldSpec {
        page: "Behavior",
        key: "speeds.walk_speed",
        label: "Walking speed",
        help: "Pixels per second.",
    },
    FieldSpec {
        page: "Behavior",
        key: "speeds.run_speed",
        label: "Running speed",
        help: "Pixels per second.",
    },
    FieldSpec {
        page: "Behavior",
        key: "speeds.charge_speed",
        label: "Charging speed",
        help: "Pixels per second.",
    },
    FieldSpec {
        page: "Behavior",
        key: "speeds.acceleration_normal",
        label: "Normal acceleration",
        help: "",
    },
    FieldSpec {
        page: "Behavior",
        key: "speeds.acceleration_charged",
        label: "Charging acceleration",
        help: "",
    },
    FieldSpec {
        page: "Behavior",
        key: "speeds.step_time_normal",
        label: "Normal step (seconds)",
        help: "",
    },
    FieldSpec {
        page: "Behavior",
        key: "speeds.step_time_charged",
        label: "Charging step (seconds)",
        help: "",
    },
    FieldSpec {
        page: "Behavior",
        key: "mud.duration_to_track_seconds",
        label: "Mud tracking (seconds)",
        help: "",
    },
    FieldSpec {
        page: "Behavior",
        key: "mud.footmark_lifetime_seconds",
        label: "Footprint lifetime (seconds)",
        help: "",
    },
    FieldSpec {
        page: "Behavior",
        key: "mud.footmark_shrink_seconds",
        label: "Footprint fade (seconds)",
        help: "",
    },
    FieldSpec {
        page: "Behavior",
        key: "mouse.grab_distance",
        label: "Cursor grab distance",
        help: "",
    },
    FieldSpec {
        page: "Behavior",
        key: "mouse.drop_distance",
        label: "Cursor drop distance",
        help: "",
    },
    FieldSpec {
        page: "Behavior",
        key: "mouse.succ_time",
        label: "Cursor nab duration (seconds)",
        help: "",
    },
    FieldSpec {
        page: "Sound & manners",
        key: "audio.enabled",
        label: "Sound",
        help: "",
    },
    FieldSpec {
        page: "Sound & manners",
        key: "audio.honk",
        label: "Honks",
        help: "",
    },
    FieldSpec {
        page: "Sound & manners",
        key: "audio.bite",
        label: "Nab sounds",
        help: "",
    },
    FieldSpec {
        page: "Sound & manners",
        key: "audio.mud",
        label: "Mud sounds",
        help: "",
    },
    FieldSpec {
        page: "Sound & manners",
        key: "audio.pat",
        label: "Petting sounds",
        help: "",
    },
    FieldSpec {
        page: "Sound & manners",
        key: "schedule.quiet_hours_enabled",
        label: "Quiet hours",
        help: "",
    },
    FieldSpec {
        page: "Sound & manners",
        key: "schedule.quiet_start",
        label: "Quiet hours start",
        help: "24-hour HH:MM, in local time.",
    },
    FieldSpec {
        page: "Sound & manners",
        key: "schedule.quiet_end",
        label: "Quiet hours end",
        help: "24-hour HH:MM, in local time.",
    },
    FieldSpec {
        page: "Sound & manners",
        key: "schedule.dnd_respect",
        label: "Respect Do Not Disturb",
        help: "Depends on desktop session support; see Platform & status.",
    },
    FieldSpec {
        page: "Sound & manners",
        key: "safety.pause_on_fullscreen",
        label: "Pause for fullscreen apps",
        help: "Depends on desktop session support; see Platform & status.",
    },
    FieldSpec {
        page: "Sound & manners",
        key: "schedule.seasonal",
        label: "Seasonal effects",
        help: "",
    },
    FieldSpec {
        page: "Sound & manners",
        key: "schedule.autumn",
        label: "Autumn leaves",
        help: "",
    },
    FieldSpec {
        page: "Platform & status",
        key: "platform.wayland",
        label: "Use native Wayland",
        help: "Linux only. Restart required. Desktop capabilities vary.",
    },
];

#[derive(Serialize)]
pub(super) struct Field {
    key: &'static str,
    page: &'static str,
    label: &'static str,
    help: &'static str,
    kind: &'static str,
    value: Value,
}

pub(super) fn fields(config: &Config) -> Vec<Field> {
    let serialized = serde_json::to_value(config).expect("validated configuration serializes");
    FIELDS
        .iter()
        .map(|spec| {
            let (section, name) = spec.key.split_once('.').expect("catalogue key");
            let value = serialized[section][name].clone();
            let kind = match &value {
                Value::Bool(_) => "toggle",
                Value::Number(_) => "number",
                _ if section == "colors" => "color",
                _ => "text",
            };
            Field {
                key: spec.key,
                page: spec.page,
                label: spec.label,
                help: spec.help,
                kind,
                value,
            }
        })
        .collect()
}
