const CORS_HEADERS = {
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type, Authorization, User-Agent, x-api-key',
};

function bytesToHex(bytes) {
    return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
}

function hexToBytes(hex) {
    const bytes = new Uint8Array(hex.length / 2);
    for (let i = 0; i < bytes.length; i++) {
        bytes[i] = parseInt(hex.substr(i * 2, 2), 16);
    }
    return bytes;
}

function timingSafeEqual(a, b) {
    if (a.length !== b.length) {
        return false;
    }
    let mismatch = 0;
    for (let i = 0; i < a.length; i++) {
        mismatch |= a.charCodeAt(i) ^ b.charCodeAt(i);
    }
    return mismatch === 0;
}

async function hashLegacyPassword(password) {
    const buffer = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(password + "_KIP_SALT_2026"));
    return bytesToHex(new Uint8Array(buffer));
}

async function derivePbkdf2Hash(password, saltBytes) {
    const enc = new TextEncoder();
    const keyMaterial = await crypto.subtle.importKey(
        'raw',
        enc.encode(password),
        { name: 'PBKDF2' },
        false,
        ['deriveBits']
    );
    const keyBits = await crypto.subtle.deriveBits(
        {
            name: 'PBKDF2',
            salt: saltBytes,
            iterations: 100000,
            hash: 'SHA-256'
        },
        keyMaterial,
        256
    );
    return bytesToHex(new Uint8Array(keyBits));
}

async function createPbkdf2PasswordRecord(password) {
    const saltBytes = crypto.getRandomValues(new Uint8Array(16));
    const saltHex = bytesToHex(saltBytes);
    const hashHex = await derivePbkdf2Hash(password, saltBytes);
    return `${saltHex}:${hashHex}`;
}

async function verifyPassword(password, storedRecord) {
    if (storedRecord.includes(':')) {
        const parts = storedRecord.split(':');
        if (parts.length !== 2) {
            return false;
        }
        const saltBytes = hexToBytes(parts[0]);
        const computedHash = await derivePbkdf2Hash(password, saltBytes);
        return timingSafeEqual(computedHash, parts[1]);
    } else {
        const legacyHash = await hashLegacyPassword(password);
        return timingSafeEqual(legacyHash, storedRecord);
    }
}

function jsonResponse(data, status = 200) {
    return new Response(JSON.stringify(data), {
        status,
        headers: {
            'Content-Type': 'application/json',
            ...CORS_HEADERS
        }
    });
}

const DEFAULT_PRESETS = [
    {
        id: 'essential-fps',
        title: 'K.I.P. FPS Boost Pack',
        author: 'KIP Studio',
        description: 'Essential performance optimization mods stack.',
        preset: ['sodium', 'lithium', 'ferrite-core', 'entityculling']
    },
    {
        id: 'vanilla-plus',
        title: 'Vanilla Enhanced Experience',
        author: 'Community',
        description: 'Quality of life, visual enhancers, and fluid animations.',
        preset: ['sodium', 'iris', 'modmenu', 'appleskin', 'ambient-sounds']
    }
];

export default {
    async fetch(request, env, ctx) {
        if (request.method === 'OPTIONS') {
            return new Response(null, { headers: CORS_HEADERS });
        }

        const url = new URL(request.url);

        if (request.method === 'GET') {
            if (url.pathname.startsWith('/api/update/')) {
                const pathParts = url.pathname.replace('/api/update/', '').split('/');
                const clientVersion = pathParts[1] || '';

                const githubManifestUrl = 'https://github.com/KIP-Studio/KIP_Hub/releases/latest/download/latest.json';

                try {
                    const ghResponse = await fetch(githubManifestUrl, {
                        headers: { 'User-Agent': 'KIP-Updater-Worker' }
                    });

                    if (!ghResponse.ok) {
                        return new Response(null, { status: 204, headers: CORS_HEADERS });
                    }

                    const updateManifest = await ghResponse.json();

                    if (updateManifest.version === clientVersion) {
                        return new Response(null, { status: 204, headers: CORS_HEADERS });
                    }

                    return jsonResponse(updateManifest);
                } catch (e) {
                    return new Response(null, { status: 204, headers: CORS_HEADERS });
                }
            }

            if (url.pathname === '/api/presets') {
                if (env && env.KIP_KV) {
                    const raw = await env.KIP_KV.get('presets_list');
                    if (raw) {
                        return jsonResponse(JSON.parse(raw));
                    }
                }
                return jsonResponse(DEFAULT_PRESETS);
            }

            if (url.pathname === '/api/news') {
                return jsonResponse({
                    news: "### K.I.P. Engine v1.5.6\n- High-performance memory management.\n- Discord Rich Presence integration.\n- Mod Doctor and Shield modules active."
                });
            }

            return jsonResponse({ status: "online", version: "1.5.6000" });
        }

        if (request.method === 'POST') {
            let payload;
            try {
                payload = await request.json();
            } catch (e) {
                return jsonResponse({ success: false, msg: "Malformed JSON payload" }, 400);
            }

            const type = payload.type;

            if (type === 'kip_login') {
                const { username, password } = payload;
                if (!username || !password || typeof username !== 'string' || typeof password !== 'string') {
                    return jsonResponse({ success: false, msg: "Missing or invalid credentials" }, 400);
                }

                const cleanUser = username.trim();
                const userKey = `user:${cleanUser.toLowerCase()}`;

                if (env && env.KIP_KV) {
                    const userDataRaw = await env.KIP_KV.get(userKey);
                    if (userDataRaw) {
                        const userData = JSON.parse(userDataRaw);
                        const isValid = await verifyPassword(password, userData.passwordHash);

                        if (isValid) {
                            if (!userData.passwordHash.includes(':')) {
                                userData.passwordHash = await createPbkdf2PasswordRecord(password);
                                await env.KIP_KV.put(userKey, JSON.stringify(userData));
                            }

                            const token = `kip_token_${crypto.randomUUID()}`;
                            await env.KIP_KV.put(`token:${token}`, cleanUser, { expirationTtl: 86400 * 30 });
                            return jsonResponse({ success: true, token, username: userData.username });
                        }
                        return jsonResponse({ success: false, msg: "Invalid password" });
                    }
                    return jsonResponse({ success: false, msg: "User not found" });
                }

                const fallbackToken = `kip_local_${crypto.randomUUID()}`;
                return jsonResponse({ success: true, token: fallbackToken, username: cleanUser });
            }

            if (type === 'kip_register') {
                const { username, password, email } = payload;
                if (!username || !password || typeof username !== 'string' || typeof password !== 'string') {
                    return jsonResponse({ success: false, msg: "Missing required fields" }, 400);
                }

                const cleanUser = username.trim();
                if (cleanUser.length < 3 || cleanUser.length > 24) {
                    return jsonResponse({ success: false, msg: "Username must be between 3 and 24 characters" }, 400);
                }

                if (password.length < 6 || password.length > 128) {
                    return jsonResponse({ success: false, msg: "Password must be between 6 and 128 characters" }, 400);
                }

                const userKey = `user:${cleanUser.toLowerCase()}`;

                if (env && env.KIP_KV) {
                    const existing = await env.KIP_KV.get(userKey);
                    if (existing) {
                        return jsonResponse({ success: false, msg: "Username already taken" });
                    }

                    const pwdHash = await createPbkdf2PasswordRecord(password);

                    const userData = {
                        username: cleanUser,
                        email: typeof email === 'string' ? email.trim() : "",
                        passwordHash: pwdHash,
                        createdAt: new Date().toISOString()
                    };

                    await env.KIP_KV.put(userKey, JSON.stringify(userData));

                    const token = `kip_token_${crypto.randomUUID()}`;
                    await env.KIP_KV.put(`token:${token}`, cleanUser, { expirationTtl: 86400 * 30 });
                    return jsonResponse({ success: true, token, username: cleanUser });
                }

                const fallbackToken = `kip_local_${crypto.randomUUID()}`;
                return jsonResponse({ success: true, token: fallbackToken, username: cleanUser });
            }

            if (type === 'publish_preset') {
                const { title, author, description, preset } = payload;
                if (!title || typeof title !== 'string' || !Array.isArray(preset)) {
                    return jsonResponse({ success: false, msg: "Invalid preset payload" }, 400);
                }

                const cleanTitle = title.trim().slice(0, 64);
                const cleanAuthor = typeof author === 'string' ? author.trim().slice(0, 32) : "Anonymous";
                const cleanDesc = typeof description === 'string' ? description.trim().slice(0, 500) : "";
                const cleanPreset = preset.slice(0, 200).filter(item => typeof item === 'string');

                const newPreset = {
                    id: `preset_${crypto.randomUUID().slice(0, 8)}`,
                    title: cleanTitle,
                    author: cleanAuthor,
                    description: cleanDesc,
                    preset: cleanPreset
                };

                if (env && env.KIP_KV) {
                    let currentList = DEFAULT_PRESETS;
                    const raw = await env.KIP_KV.get('presets_list');
                    if (raw) {
                        try {
                            currentList = JSON.parse(raw);
                        } catch (e) {}
                    }
                    currentList.unshift(newPreset);
                    if (currentList.length > 50) {
                        currentList = currentList.slice(0, 50);
                    }
                    await env.KIP_KV.put('presets_list', JSON.stringify(currentList));
                }

                return jsonResponse({ success: true, preset: newPreset });
            }

            if (type === 'bug_report') {
                const { report, telemetry, version } = payload;
                if (!report || typeof report !== 'string') {
                    return jsonResponse({ success: false, msg: "Report content is required" }, 400);
                }

                const reportId = `report_${Date.now()}_${crypto.randomUUID().slice(0, 6)}`;

                if (env && env.KIP_KV) {
                    const record = {
                        report: report.slice(0, 10000),
                        telemetry: typeof telemetry === 'string' ? telemetry.slice(0, 2000) : "",
                        version: typeof version === 'string' ? version.slice(0, 32) : "unknown",
                        timestamp: new Date().toISOString()
                    };
                    await env.KIP_KV.put(`bug:${reportId}`, JSON.stringify(record), { expirationTtl: 86400 * 90 });
                }

                return jsonResponse({ success: true, id: reportId });
            }

            return jsonResponse({ success: false, msg: "Unknown payload type" }, 400);
        }

        return new Response("Not Found", { status: 404, headers: CORS_HEADERS });
    }
};