//! The recommended fonts at pinned versions, with the SHA-256 of each file (FONT-1, FONT-3, FONT-7,
//! FONT-8, FONT-11). Atkinson Hyperlegible Mono and iA Writer Mono have no releases: their files
//! come from a pinned commit.
//! A new version of a font is a new pin here, with the new checksums.
use super::{Font, Origin, Part};

const JETBRAINS_ZIP: &str = "https://github.com/JetBrains/JetBrainsMono/releases/download/v2.304/JetBrainsMono-2.304.zip";
const JETBRAINS: &[Part] = &[
    Part { origin: Origin::Zip { url: JETBRAINS_ZIP, member: "fonts/ttf/JetBrainsMono-Regular.ttf" }, file: "JetBrainsMono-Regular.ttf", sha256: "a0bf60ef0f83c5ed4d7a75d45838548b1f6873372dfac88f71804491898d138f" },
    Part { origin: Origin::Zip { url: JETBRAINS_ZIP, member: "fonts/ttf/JetBrainsMono-Bold.ttf" }, file: "JetBrainsMono-Bold.ttf", sha256: "5590990c82e097397517f275f430af4546e1c45cff408bde4255dad142479dcb" },
    Part { origin: Origin::Zip { url: JETBRAINS_ZIP, member: "fonts/ttf/JetBrainsMono-Italic.ttf" }, file: "JetBrainsMono-Italic.ttf", sha256: "9d0a1f7a708e6af183f1193b7e81d40da294f5c67682c085d8401c60aac8ded4" },
    Part { origin: Origin::Zip { url: JETBRAINS_ZIP, member: "fonts/ttf/JetBrainsMono-BoldItalic.ttf" }, file: "JetBrainsMono-BoldItalic.ttf", sha256: "4039d5ce0ed225bf9c8b2c8c6436290ae2f356b7e90d70fa666227238324aa3b" },
    Part { origin: Origin::Zip { url: JETBRAINS_ZIP, member: "OFL.txt" }, file: "JetBrainsMono-LICENSE.txt", sha256: "30f0c136e3c88e422d0791acd97238870f9054a9729bc34cf2ff0d4ed8cac4ad" },
];

const ATKINSON: &[Part] = &[
    Part { origin: Origin::Raw("https://raw.githubusercontent.com/googlefonts/atkinson-hyperlegible-next-mono/154d50362016cc3e873eb21d242cd0772384c8f9/fonts/ttf/AtkinsonHyperlegibleMono-Regular.ttf"), file: "AtkinsonHyperlegibleMono-Regular.ttf", sha256: "2c9a7b9fabebd5c746a808d8a1ef2f52e618850caad6ec4fd9f5c48f56e1c1af" },
    Part { origin: Origin::Raw("https://raw.githubusercontent.com/googlefonts/atkinson-hyperlegible-next-mono/154d50362016cc3e873eb21d242cd0772384c8f9/fonts/ttf/AtkinsonHyperlegibleMono-Bold.ttf"), file: "AtkinsonHyperlegibleMono-Bold.ttf", sha256: "51ad68fafd7ec68051f64983c9c23182a7084268b5cdb0a154dbd53ade079da6" },
    Part { origin: Origin::Raw("https://raw.githubusercontent.com/googlefonts/atkinson-hyperlegible-next-mono/154d50362016cc3e873eb21d242cd0772384c8f9/fonts/ttf/AtkinsonHyperlegibleMono-Italic.ttf"), file: "AtkinsonHyperlegibleMono-Italic.ttf", sha256: "356060ab70ae62e4460dd824ec1b734af24e5c3355a8302700151d35d1382dd4" },
    Part { origin: Origin::Raw("https://raw.githubusercontent.com/googlefonts/atkinson-hyperlegible-next-mono/154d50362016cc3e873eb21d242cd0772384c8f9/fonts/ttf/AtkinsonHyperlegibleMono-BoldItalic.ttf"), file: "AtkinsonHyperlegibleMono-BoldItalic.ttf", sha256: "1286feda7848b1078576f2293201dff9428dad309c63220ed0825b771994a9a6" },
    Part { origin: Origin::Raw("https://raw.githubusercontent.com/googlefonts/atkinson-hyperlegible-next-mono/154d50362016cc3e873eb21d242cd0772384c8f9/OFL.txt"), file: "AtkinsonHyperlegibleMono-LICENSE.txt", sha256: "1ebb31cf7393164f20d10c1d48406cddb5314feff8465531cf1e4ba37e9dd740" },
];

const PLEX_ZIP: &str = "https://github.com/IBM/plex/releases/download/%40ibm%2Fplex-mono%402.5.0/ibm-plex-mono.zip";
const PLEX: &[Part] = &[
    Part { origin: Origin::Zip { url: PLEX_ZIP, member: "ibm-plex-mono/fonts/complete/ttf/IBMPlexMono-Regular.ttf" }, file: "IBMPlexMono-Regular.ttf", sha256: "7c6fbddca4b700be918f5f6183d9bd4464fa427fe435f0b480d77fe2bb8c5a43" },
    Part { origin: Origin::Zip { url: PLEX_ZIP, member: "ibm-plex-mono/fonts/complete/ttf/IBMPlexMono-Bold.ttf" }, file: "IBMPlexMono-Bold.ttf", sha256: "74e5eedcfa4596497d34e19023cabdabd3a8c852b903007a5654a59591a72ffb" },
    Part { origin: Origin::Zip { url: PLEX_ZIP, member: "ibm-plex-mono/fonts/complete/ttf/IBMPlexMono-Italic.ttf" }, file: "IBMPlexMono-Italic.ttf", sha256: "a55368deb953f594ac18336a1029009ed820e2d9abbb57ad601f61e22bec072b" },
    Part { origin: Origin::Zip { url: PLEX_ZIP, member: "ibm-plex-mono/fonts/complete/ttf/IBMPlexMono-BoldItalic.ttf" }, file: "IBMPlexMono-BoldItalic.ttf", sha256: "c8b00068fd2fd69bbcf62059148623f09d19e9ada8bf9e8caf1aedd01347046a" },
    Part { origin: Origin::Zip { url: PLEX_ZIP, member: "ibm-plex-mono/LICENSE.txt" }, file: "IBMPlexMono-LICENSE.txt", sha256: "7e6b2818edbd8f6a01ae80641cc8f16a51080d08fb4e532be3a0b6f74adb07da" },
];

const IOSEVKA_ZIP: &str = "https://github.com/be5invis/Iosevka/releases/download/v34.9.1/PkgTTF-Iosevka-34.9.1.zip";
const IOSEVKA: &[Part] = &[
    Part { origin: Origin::Zip { url: IOSEVKA_ZIP, member: "Iosevka-Regular.ttf" }, file: "Iosevka-Regular.ttf", sha256: "1ba24b81d82df65a68c9d0bb44bda306fdcdb5d7763afe89b0dea29ea827d1a2" },
    Part { origin: Origin::Zip { url: IOSEVKA_ZIP, member: "Iosevka-Bold.ttf" }, file: "Iosevka-Bold.ttf", sha256: "1988973571c96725d240c97bd2131b5deecdedfc537ffd30e11aff6ec0286488" },
    Part { origin: Origin::Zip { url: IOSEVKA_ZIP, member: "Iosevka-Italic.ttf" }, file: "Iosevka-Italic.ttf", sha256: "8bb76fdd0bda08193781e9f1703e4021d3afca6b48a3e7bbb61b699efe989115" },
    Part { origin: Origin::Zip { url: IOSEVKA_ZIP, member: "Iosevka-BoldItalic.ttf" }, file: "Iosevka-BoldItalic.ttf", sha256: "13579582b40296b7b79200c3d419d99de2c8b1ef051ec92c9fbc1df841991bec" },
    Part { origin: Origin::Raw("https://raw.githubusercontent.com/be5invis/Iosevka/v34.9.1/LICENSE.md"), file: "Iosevka-LICENSE.md", sha256: "4ba53c7c1cb39279aae5f8d7d22054c485c71169920e5a36ed098b115e2e3c5d" },
];

const IOSEVKA_TERM_ZIP: &str = "https://github.com/be5invis/Iosevka/releases/download/v34.9.1/PkgTTF-IosevkaTerm-34.9.1.zip";
const IOSEVKA_TERM: &[Part] = &[
    Part { origin: Origin::Zip { url: IOSEVKA_TERM_ZIP, member: "IosevkaTerm-Regular.ttf" }, file: "IosevkaTerm-Regular.ttf", sha256: "430ad4f56c6cd82e4c954efb6dd4b3e2b4cc2b2b1bd4f8143f3d3800096f32b4" },
    Part { origin: Origin::Zip { url: IOSEVKA_TERM_ZIP, member: "IosevkaTerm-Bold.ttf" }, file: "IosevkaTerm-Bold.ttf", sha256: "de8fa430d7ab8f86a3232b2cef1f9a0d2dd272b2e18810b203bd55d36aa8a9d1" },
    Part { origin: Origin::Zip { url: IOSEVKA_TERM_ZIP, member: "IosevkaTerm-Italic.ttf" }, file: "IosevkaTerm-Italic.ttf", sha256: "bec34a212454648ee3c2dcbb8ff29a97725cb23abaff782b8c1a6f77436cb66f" },
    Part { origin: Origin::Zip { url: IOSEVKA_TERM_ZIP, member: "IosevkaTerm-BoldItalic.ttf" }, file: "IosevkaTerm-BoldItalic.ttf", sha256: "6679ec0416e8bab735eb70de77d3622aaed775db62286e3a77ff3a09cc63b8bf" },
    Part { origin: Origin::Raw("https://raw.githubusercontent.com/be5invis/Iosevka/v34.9.1/LICENSE.md"), file: "IosevkaTerm-LICENSE.md", sha256: "4ba53c7c1cb39279aae5f8d7d22054c485c71169920e5a36ed098b115e2e3c5d" },
];

const FIRA_ZIP: &str = "https://github.com/tonsky/FiraCode/releases/download/6.2/Fira_Code_v6.2.zip";
const FIRA: &[Part] = &[
    Part { origin: Origin::Zip { url: FIRA_ZIP, member: "ttf/FiraCode-Regular.ttf" }, file: "FiraCode-Regular.ttf", sha256: "5992ab9640e2df491b2f609467b1de60e8bc39b2c28db184342a0592d98f6117" },
    Part { origin: Origin::Zip { url: FIRA_ZIP, member: "ttf/FiraCode-Bold.ttf" }, file: "FiraCode-Bold.ttf", sha256: "41f6554e845e2f5b70adad3950122334b866aac436793b7742ade600067701be" },
    Part { origin: Origin::Raw("https://raw.githubusercontent.com/tonsky/FiraCode/6.2/LICENSE"), file: "FiraCode-LICENSE.txt", sha256: "1d41e10031ab125302780a05ec4c91d218e47db0c7e37cf315cce5e608cdc25c" },
];

const CASCADIA_ZIP: &str = "https://github.com/microsoft/cascadia-code/releases/download/v2407.24/CascadiaCode-2407.24.zip";
const CASCADIA: &[Part] = &[
    Part { origin: Origin::Zip { url: CASCADIA_ZIP, member: "ttf/static/CascadiaCode-Regular.ttf" }, file: "CascadiaCode-Regular.ttf", sha256: "c33ef522cdfeff99907fb54f3e97152bb18bfb9b56ec2fef4d4ceec51c8974a4" },
    Part { origin: Origin::Zip { url: CASCADIA_ZIP, member: "ttf/static/CascadiaCode-Bold.ttf" }, file: "CascadiaCode-Bold.ttf", sha256: "58197c5c3ba2967d4780b0ef8aee72d0cbbfa42885ddede9ec96fa7a1df5aead" },
    Part { origin: Origin::Zip { url: CASCADIA_ZIP, member: "ttf/static/CascadiaCode-Italic.ttf" }, file: "CascadiaCode-Italic.ttf", sha256: "1f0676937792fe550effeafb7f4622080c2a5d62715d1245d471258fa1a23b8c" },
    Part { origin: Origin::Zip { url: CASCADIA_ZIP, member: "ttf/static/CascadiaCode-BoldItalic.ttf" }, file: "CascadiaCode-BoldItalic.ttf", sha256: "aff1984cbbdd990137691ae9567b87e7b1190f6a4e146017c6a0893ca4b000e7" },
    Part { origin: Origin::Raw("https://raw.githubusercontent.com/microsoft/cascadia-code/v2407.24/LICENSE"), file: "CascadiaCode-LICENSE.txt", sha256: "51882cd3cdba4e16f220f44ddb08a635c38c44ea6e0975db2574f4be6f958238" },
];

const IA_WRITER: &[Part] = &[
    Part { origin: Origin::Raw("https://raw.githubusercontent.com/iaolo/iA-Fonts/c6588670c71e9ac628acc27b72cde4bf12726b7f/iA%20Writer%20Mono/Static/iAWriterMonoS-Regular.ttf"), file: "iAWriterMonoS-Regular.ttf", sha256: "929605302a57250e712908cb5f6e1ce80c7d0accd5fd2555345f29a5e8d4e30b" },
    Part { origin: Origin::Raw("https://raw.githubusercontent.com/iaolo/iA-Fonts/c6588670c71e9ac628acc27b72cde4bf12726b7f/iA%20Writer%20Mono/Static/iAWriterMonoS-Bold.ttf"), file: "iAWriterMonoS-Bold.ttf", sha256: "76aa5b5b4f9091a9c686a2a6fe5ff3495bb791994d7079857e5b24ae98063743" },
    Part { origin: Origin::Raw("https://raw.githubusercontent.com/iaolo/iA-Fonts/c6588670c71e9ac628acc27b72cde4bf12726b7f/iA%20Writer%20Mono/Static/iAWriterMonoS-Italic.ttf"), file: "iAWriterMonoS-Italic.ttf", sha256: "c7e7e36e8167bf50f27e46c9cab447d04cc570bd388b998044e2e29f4cebc216" },
    Part { origin: Origin::Raw("https://raw.githubusercontent.com/iaolo/iA-Fonts/c6588670c71e9ac628acc27b72cde4bf12726b7f/iA%20Writer%20Mono/Static/iAWriterMonoS-BoldItalic.ttf"), file: "iAWriterMonoS-BoldItalic.ttf", sha256: "b0cf9571234528b0896aacf97eb3ec45712da40b8410c799fa43ea123bc19e35" },
    Part { origin: Origin::Raw("https://raw.githubusercontent.com/iaolo/iA-Fonts/c6588670c71e9ac628acc27b72cde4bf12726b7f/iA%20Writer%20Mono/LICENSE.md"), file: "iAWriterMonoS-LICENSE.md", sha256: "2eb84d6d03a9af6e99816f82f50a77c26e7ff6681293f4619cd33a392a8c13b6" },
];

/// The recommended fonts, in the order of the tip and of the request to the agent.
pub const FONTS: &[Font] = &[
    Font { name: "JetBrains Mono", family: "JetBrains Mono", prefix: "JetBrainsMono", parts: JETBRAINS },
    Font { name: "Atkinson Hyperlegible Mono", family: "Atkinson Hyperlegible Mono", prefix: "AtkinsonHyperlegibleMono", parts: ATKINSON },
    Font { name: "IBM Plex Mono", family: "IBM Plex Mono", prefix: "IBMPlexMono", parts: PLEX },
    Font { name: "Iosevka", family: "Iosevka", prefix: "Iosevka", parts: IOSEVKA },
    Font { name: "Iosevka Term", family: "Iosevka Term", prefix: "IosevkaTerm", parts: IOSEVKA_TERM },
    Font { name: "Fira Code", family: "Fira Code", prefix: "FiraCode", parts: FIRA },
    Font { name: "Cascadia Code", family: "Cascadia Code", prefix: "CascadiaCode", parts: CASCADIA },
    Font { name: "iA Writer Mono", family: "iA Writer Mono S", prefix: "iAWriterMonoS", parts: IA_WRITER },
];
