use topcoat::{
    Result,
    router::page,
    view::{component, view},
};

#[page("/")]
async fn home() -> Result {
    view! {
        <div>
            <h2 class="text-2xl font-semibold tracking-tight">"Dashboard"</h2>
            <p class="text-zinc-400 mt-1 text-sm">"Overview of your proxy infrastructure"</p>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mt-6">
                stat_card(id: "stat-users", label: "Total Users")
                stat_card(id: "stat-sessions", label: "Active Sessions")
                stat_card(id: "stat-bandwidth", label: "Total Bandwidth")
                stat_card(id: "stat-carriers", label: "Carriers")
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mt-8">
                endpoint_status()
                quick_connect()
            </div>
        </div>

        <script>
            "const API = window.__API_URL || 'http://localhost:3001';"
            "fetch(API + '/api/users').then(r => r.json()).then(d => {"
            "  document.getElementById('stat-users').textContent = d.length;"
            "}).catch(() => {});"
            "fetch(API + '/api/sessions').then(r => r.json()).then(d => {"
            "  document.getElementById('stat-sessions').textContent = d.filter(s => s.active).length;"
            "  const bytes = d.reduce((a, s) => a + s.bytes_up + s.bytes_down, 0);"
            "  document.getElementById('stat-bandwidth').textContent = formatBytes(bytes);"
            "}).catch(() => {});"
            "fetch(API + '/api/carriers').then(r => r.json()).then(d => {"
            "  const online = d.filter(c => c.online).length;"
            "  document.getElementById('stat-carriers').textContent = online > 0 ? online + ' online' : 'Direct only';"
            "}).catch(() => {});"
            "function formatBytes(b) {"
            "  if (b === 0) return '0 B';"
            "  const u = ['B','KB','MB','GB','TB'];"
            "  const i = Math.floor(Math.log(b) / Math.log(1024));"
            "  return (b / Math.pow(1024, i)).toFixed(1) + ' ' + u[i];"
            "}"
        </script>
    }
}

#[component]
async fn stat_card(id: &str, label: &str) -> Result {
    view! {
        <div class="rounded-lg border border-zinc-800 bg-zinc-900 p-5">
            <p class="text-xs font-medium text-zinc-500 uppercase tracking-wider">(label)</p>
            <p class="text-2xl font-semibold mt-2" id=(id)>"--"</p>
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
                info_row(label: "Host", value: "parallax.stynx.app")
                info_row(label: "Port", value: "1080")
                info_row(label: "Location", value: "Hostinger VPS, Indonesia")
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
async fn info_row(label: &str, value: &str) -> Result {
    view! {
        <div class="flex items-center justify-between py-2 border-b border-zinc-800/50">
            <span class="text-zinc-500">(label)</span>
            <code class="text-emerald-400 text-xs bg-zinc-800 px-2 py-1 rounded">(value)</code>
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
            </div>
        </div>
    }
}
