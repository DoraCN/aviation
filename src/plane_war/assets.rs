use bevy::prelude::*;

// ── 嵌入资源（编译期打包，不依赖工作目录） ──

pub const FIGHTER_BYTES: &[u8] = include_bytes!("../../assets/images/fighter.png");
pub const FIGHTER_BOMB_BYTES: &[u8] = include_bytes!("../../assets/images/fighter-bomb.png");
pub const BOMB_BYTES: &[u8] = include_bytes!("../../assets/images/bomb.png");

pub const ENEMY_64_BYTES: &[(&str, &[u8])] = &[
    ("alien", include_bytes!("../../assets/images/64/alien.png")),
    ("cards", include_bytes!("../../assets/images/64/cards.png")),
    (
        "caterpillar",
        include_bytes!("../../assets/images/64/caterpillar.png"),
    ),
    (
        "clown-face",
        include_bytes!("../../assets/images/64/clown-face.png"),
    ),
    (
        "exploding-head",
        include_bytes!("../../assets/images/64/exploding-head.png"),
    ),
    (
        "face-with-steam-from-nose",
        include_bytes!("../../assets/images/64/face-with-steam-from-nose.png"),
    ),
    ("gift", include_bytes!("../../assets/images/64/gift.png")),
    ("goal", include_bytes!("../../assets/images/64/goal.png")),
    (
        "partying-face",
        include_bytes!("../../assets/images/64/partying-face.png"),
    ),
    (
        "pile-of-poo",
        include_bytes!("../../assets/images/64/pile-of-poo.png"),
    ),
    (
        "see-no-evil-monkey",
        include_bytes!("../../assets/images/64/see-no-evil-monkey.png"),
    ),
    (
        "thinking-face",
        include_bytes!("../../assets/images/64/thinking-face.png"),
    ),
    ("ufo", include_bytes!("../../assets/images/64/ufo.png")),
];

pub const ENEMY_100_BYTES: &[(&str, &[u8])] = &[
    (
        "chatbot",
        include_bytes!("../../assets/images/100/chatbot.png"),
    ),
    (
        "dollar-bag",
        include_bytes!("../../assets/images/100/dollar-bag.png"),
    ),
    (
        "robotic",
        include_bytes!("../../assets/images/100/robotic.png"),
    ),
    (
        "star-struck",
        include_bytes!("../../assets/images/100/star-struck.png"),
    ),
    ("trust", include_bytes!("../../assets/images/100/trust.png")),
];

pub const BG_MUSIC_BYTES: &[u8] = include_bytes!("../../assets/media/bg.mp3");
pub const BOMB_SFX_BYTES: &[u8] = include_bytes!("../../assets/media/bomb.mp3");

/// 解码 PNG → `Handle<Image>`。
pub fn decode_image(bytes: &[u8], images: &mut Assets<Image>) -> Handle<Image> {
    let image = Image::from_buffer(
        bytes,
        bevy::image::ImageType::Format(bevy::image::ImageFormat::Png),
        bevy::image::CompressedImageFormats::NONE,
        true,
        bevy::image::ImageSampler::Default,
        bevy::asset::RenderAssetUsages::default(),
    )
    .expect("failed to decode embedded PNG");
    images.add(image)
}

/// 构造 `AudioSource`（MP3，Bevy 默认特性已包含解码器）。
pub fn audio_source(bytes: &[u8]) -> bevy::audio::AudioSource {
    bevy::audio::AudioSource {
        bytes: bytes.to_vec().into(),
    }
}
