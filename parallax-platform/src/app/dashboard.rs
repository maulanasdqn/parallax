use topcoat::{
    Result,
    context::{Cx, app_context},
    router::page,
    view::{component, view},
};

use crate::db::Database;

#[page("/")]
async fn home(cx: &Cx) -> Result {
    let db = app_context::<Database>(cx);
    let user_count = db.user_count().await.to_string();
    let session_count = db.active_session_count().await.to_string();
    let bw = format_bytes(db.total_bandwidth().await as u64);
    let carrier_count = db.carrier_count().await;
    let carrier_text = if carrier_count == 0 {
        "Direct only".to_string()
    } else {
        format!("{carrier_count} online")
    };

    view! {
        <div>
            <h2 class="text-2xl font-semibold tracking-tight">"Dashboard"</h2>
            <p class="text-zinc-400 mt-1 text-sm">"Overview of your proxy infrastructure"</p>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mt-6">
                stat_card(label: "Total Users", value: &user_count)
                stat_card(label: "Active Sessions", value: &session_count)
                stat_card(label: "Total Bandwidth", value: &bw)
                stat_card(label: "Carriers", value: &carrier_text)
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mt-8">
                endpoint_status()
                quick_connect()
            </div>
        </div>
    }
}

#[component]
async fn stat_card(label: &str, value: &str) -> Result {
    view! {
        <div class="rounded-lg border border-zinc-800 bg-zinc-900 p-5">
            <p class="text-xs font-medium text-zinc-500 uppercase tracking-wider">(label)</p>
            <p class="text-2xl font-semibold mt-2">(value)</p>
        </div>
    }
}

#[component]
async fn endpoint_status() -> Result {
    view! {
        <div class="rounded-lg border border-zinc-800 bg-zinc-900">
            <div class="px-5 py-4 border-b border-zinc-800">
                <h3 class="text-sm font-semibold">"Proxy Endpoint"</h3>
            </div>
            <div class="p-5 space-y-3 text-sm">
                <div class="flex items-center justify-between py-2 border-b border-zinc-800/50">
                    <span class="text-zinc-500">"Host"</span>
                    <code class="text-emerald-400 text-xs bg-zinc-800 px-2 py-1 rounded">"parallax.stynx.app"</code>
                </div>
                <div class="flex items-center justify-between py-2 border-b border-zinc-800/50">
                    <span class="text-zinc-500">"Port"</span>
                    <code class="text-emerald-400 text-xs bg-zinc-800 px-2 py-1 rounded">"1080"</code>
                </div>
                <div class="flex items-center justify-between py-2 border-b border-zinc-800/50">
                    <span class="text-zinc-500">"Location"</span>
                    <span class="text-zinc-300">"Hostinger VPS, Indonesia"</span>
                </div>
                <div class="flex items-center justify-between py-2">
                    <span class="text-zinc-500">"Status"</span>
                    <div class="flex items-center gap-2">
                        <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
                        <span class="text-emerald-400">"Online"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
async fn quick_connect() -> Result {
    view! {
        <div class="rounded-lg border border-zinc-800 bg-zinc-900">
            <div class="px-5 py-4 border-b border-zinc-800">
                <h3 class="text-sm font-semibold">"Quick Connect"</h3>
            </div>
            <div class="p-5 space-y-4">
                <div>
                    <p class="text-xs text-zinc-500 uppercase tracking-wider mb-2">"SOCKS5"</p>
                    <code class="block text-sm bg-zinc-800 rounded px-3 py-2 text-emerald-400 overflow-x-auto">
                        "socks5h://user:apikey@parallax.stynx.app:1080"
                    </code>
                </div>
                <div>
                    <p class="text-xs text-zinc-500 uppercase tracking-wider mb-2">"HTTP Proxy"</p>
                    <code class="block text-sm bg-zinc-800 rounded px-3 py-2 text-emerald-400 overflow-x-auto">
                        "http://user:apikey@parallax.stynx.app:1080"
                    </code>
                </div>
                <div>
                    <p class="text-xs text-zinc-500 uppercase tracking-wider mb-2">"curl"</p>
                    <code class="block text-sm bg-zinc-800 rounded px-3 py-2 text-zinc-300 overflow-x-auto">
                        "curl -x socks5h://user:apikey@parallax.stynx.app:1080 https://httpbin.org/ip"
                    </code>
                </div>
                <p class="text-xs text-zinc-600 mt-2">
                    "Replace user and apikey with credentials from the Users page."
                </p>
            </div>
        </div>
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".into();
    }
    let units = ["B", "KB", "MB", "GB", "TB"];
    let i = (bytes as f64).log(1024.0).floor() as usize;
    let val = bytes as f64 / 1024_f64.powi(i as i32);
    format!("{val:.1} {}", units[i.min(units.len() - 1)])
}
