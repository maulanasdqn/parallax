use topcoat::{
    Result,
    router::page,
    view::{component, view},
};

#[page("/proxies")]
async fn proxies_page() -> Result {
    view! {
        <div>
            <h2 class="text-2xl font-semibold tracking-tight">"Proxies"</h2>
            <p class="text-zinc-400 mt-1 text-sm">"Manage proxy endpoints and carriers"</p>

            <div class="mt-6 rounded-lg border border-zinc-800 bg-zinc-900">
                <div class="px-5 py-4 border-b border-zinc-800 flex items-center justify-between">
                    <h3 class="text-sm font-semibold">"Endpoints"</h3>
                    <span class="text-xs text-zinc-500">"1 endpoint active"</span>
                </div>
                <table class="w-full text-sm">
                    <thead>
                        <tr class="border-b border-zinc-800 text-left text-xs text-zinc-500 uppercase tracking-wider">
                            <th class="px-5 py-3">"Name"</th>
                            <th class="px-5 py-3">"Type"</th>
                            <th class="px-5 py-3">"Host"</th>
                            <th class="px-5 py-3">"IP"</th>
                            <th class="px-5 py-3">"Status"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-zinc-800">
                        <tr class="hover:bg-zinc-800/50">
                            <td class="px-5 py-3 font-medium">"Hostinger VPS"</td>
                            <td class="px-5 py-3">
                                <span class="text-xs px-2 py-1 rounded bg-blue-900 text-blue-400">"Datacenter"</span>
                            </td>
                            <td class="px-5 py-3">
                                <code class="text-xs bg-zinc-800 px-2 py-1 rounded text-zinc-300">"parallax.stynx.app:1080"</code>
                            </td>
                            <td class="px-5 py-3 font-mono text-xs text-zinc-400">"72.62.125.38"</td>
                            <td class="px-5 py-3">
                                <div class="flex items-center gap-2">
                                    <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
                                    <span class="text-xs text-emerald-400">"Online"</span>
                                </div>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>

            <div class="mt-6 rounded-lg border border-zinc-800 bg-zinc-900">
                <div class="px-5 py-4 border-b border-zinc-800 flex items-center justify-between">
                    <h3 class="text-sm font-semibold">"Mobile Carriers"</h3>
                    <span class="text-xs text-zinc-500" id="carrier-count">"loading..."</span>
                </div>
                <tbody id="carriers-body"></tbody>
                <div id="carriers-empty" class="p-8 text-center">
                    <p class="text-zinc-500 text-sm">"No mobile carriers registered"</p>
                    <p class="text-zinc-600 text-xs mt-2">"Connect USB 4G modems to add mobile proxy endpoints."</p>
                </div>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mt-6">
                connection_guide()
                protocols_card()
            </div>
        </div>

        <script>
            "const API = window.__API_URL || 'http://localhost:3001';"
            "fetch(API + '/api/carriers').then(r => r.json()).then(carriers => {"
            "  document.getElementById('carrier-count').textContent = carriers.length + ' registered';"
            "  if (carriers.length === 0) return;"
            "  document.getElementById('carriers-empty').classList.add('hidden');"
            "}).catch(() => {});"
        </script>
    }
}

#[component]
async fn connection_guide() -> Result {
    view! {
        <div class="rounded-lg border border-zinc-800 bg-zinc-900">
            <div class="px-5 py-4 border-b border-zinc-800">
                <h3 class="text-sm font-semibold">"Connection Guide"</h3>
            </div>
            <div class="p-5 space-y-4 text-sm">
                <div>
                    <p class="text-xs text-zinc-500 uppercase tracking-wider mb-2">"Browser (Firefox)"</p>
                    <p class="text-zinc-400">"Settings > Network > Manual Proxy > SOCKS Host: parallax.stynx.app, Port: 1080, SOCKS v5"</p>
                </div>
                <div>
                    <p class="text-xs text-zinc-500 uppercase tracking-wider mb-2">"Python (requests)"</p>
                    <code class="block bg-zinc-800 rounded px-3 py-2 text-zinc-300 overflow-x-auto">
                        "proxies = {'https': 'socks5h://user:key@parallax.stynx.app:1080'}"
                    </code>
                </div>
                <div>
                    <p class="text-xs text-zinc-500 uppercase tracking-wider mb-2">"Node.js"</p>
                    <code class="block bg-zinc-800 rounded px-3 py-2 text-zinc-300 overflow-x-auto">
                        "new SocksProxyAgent('socks5h://user:key@parallax.stynx.app:1080')"
                    </code>
                </div>
            </div>
        </div>
    }
}

#[component]
async fn protocols_card() -> Result {
    view! {
        <div class="rounded-lg border border-zinc-800 bg-zinc-900">
            <div class="px-5 py-4 border-b border-zinc-800">
                <h3 class="text-sm font-semibold">"Supported Protocols"</h3>
            </div>
            <div class="p-5 space-y-3">
                protocol_row(name: "SOCKS5", desc: "Full TCP proxy with server-side DNS")
                protocol_row(name: "HTTP CONNECT", desc: "HTTPS tunneling through proxy")
                protocol_row(name: "HTTP Forward", desc: "Plain HTTP request forwarding")
            </div>
        </div>
    }
}

#[component]
async fn protocol_row(name: &str, desc: &str) -> Result {
    view! {
        <div class="flex items-center justify-between py-2 border-b border-zinc-800/50">
            <div class="flex items-center gap-3">
                <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
                <span class="font-medium text-sm">(name)</span>
            </div>
            <span class="text-xs text-zinc-500">(desc)</span>
        </div>
    }
}
