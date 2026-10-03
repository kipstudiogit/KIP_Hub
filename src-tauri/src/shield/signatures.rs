use regex::bytes::Regex as BytesRegex;

pub struct ThreatSignatures {
    pub webhook_pattern: BytesRegex,
    pub b64_pattern: BytesRegex,
    pub telegram_bot_pattern: BytesRegex,
    pub pastebin_pattern: BytesRegex,
    pub dangerous_apis: Vec<&'static [u8]>,
    pub stealer_targets: Vec<&'static [u8]>,
    pub rce_commands: Vec<&'static [u8]>,
    pub whitelist_hashes: Vec<&'static str>,
    pub ignored_prefixes: Vec<&'static str>,
}

impl ThreatSignatures {
    pub fn new() -> Self {
        Self {
            webhook_pattern: BytesRegex::new(r"https?://(?:ptb\.|canary\.)?discord(?:app)?\.com/api/webhooks/\d+/[a-zA-Z0-9_\-]+").unwrap(),
            b64_pattern: BytesRegex::new(r"(?:[A-Za-z0-9+/]{4}){10,}(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?").unwrap(),
            telegram_bot_pattern: BytesRegex::new(r"https?://api\.telegram\.org/bot\d+:[a-zA-Z0-9_\-]+").unwrap(),
            pastebin_pattern: BytesRegex::new(r"https?://(?:pastebin\.com/raw|hastebin\.com/raw|ghostbin\.com/raw)/[a-zA-Z0-9_\-]+").unwrap(),
            dangerous_apis: vec![
                b"java/lang/Runtime.getRuntime()Ljava/lang/Runtime;",
                b"java/lang/Runtime.exec",
                b"java/lang/ProcessBuilder",
                b"sun/misc/Unsafe",
                b"java/lang/ClassLoader.defineClass",
                b"java/lang/instrument/Instrumentation",
                b"java/lang/reflect/Method.invoke",
                b"java/net/URLClassLoader.<init>",
            ],
            stealer_targets: vec![
                b"Login Data",
                b"launcher_accounts.json",
                b"usercache.json",
                b"Essential/credentials.json",
                b"feather/accounts.json",
                b"FileZilla/recentservers.xml",
                b"AppData/Local/Google/Chrome/User Data",
                b"AppData/Roaming/Opera Software",
                b"AppData/Local/BraveSoftware/Brave-Browser",
                b"AppData/Roaming/Mozilla/Firefox/Profiles",
                b"Discord/Local Storage/leveldb",
                b"solana_id.json",
                b"nkbihfbeogaeaoehlefnkodbefgpgknn",
            ],
            rce_commands: vec![
                b"powershell -enc",
                b"powershell.exe -ExecutionPolicy Bypass",
                b"cmd.exe /c start",
                b"/bin/sh -i",
                b"curl -s -O",
                b"certutil -urlcache",
                b"bitsadmin /transfer",
            ],
            whitelist_hashes: vec![
                "2a98f4cd5c95738ab8243302636fa0982bbfe3b52d9a5b3a4a5b6c7d8e9f0a1b",
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ],
            ignored_prefixes: vec![
                "kotlin/",
                "kotlinx/",
                "org/spongepowered/",
                "org/objectweb/asm/",
                "it/unimi/dsi/fastutil/",
                "io/netty/",
                "com/google/",
                "com/mojang/",
                "net/minecraft/",
                "javax/",
            ],
        }
    }
}