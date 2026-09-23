use topcoat::{
    Result,
    router::page,
    view::{component, view},
};

#[page("/")]
async fn home() -> Result {
    view! {
        <div class="max-w-6xl mx-auto px-6 py-12">
            <header class="mb-12">
                <h1 class="text-3xl font-bold tracking-tight">"Parallax"</h1>
                <p class="text-zinc-400 mt-2">"Mobile Proxy Platform"</p>
            </header>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                stat_card(label: "Active Sessions", value: "0")
                stat_card(label: "Bandwidth Used", value: "0 MB")
                stat_card(label: "Carriers Online", value: "0")
            </div>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mt-8">
                proxy_config_card()
                carriers_card()
            </div>
        </div>
    }
}

#[component]
async fn stat_card(label: &str, value: &str) -> Result {
    view! {
        <div class="rounded-lg border border-zinc-800 bg-zinc-900 p-6">
            <p class="text-sm text-zinc-400">(label)</p>
            <p class="text-2xl font-semibold mt-1">(value)</p>
        </div>
    }
}

#[component]
async fn proxy_config_card() -> Result {
    view! {
        <div class="rounded-lg border border-zinc-800 bg-zinc-900 p-6">
            <h2 class="text-lg font-semibold mb-4">"Proxy Configuration"</h2>
            <div class="space-y-3 text-sm">
                config_row(label: "SOCKS5", value: "parallax.stynx.app:1080")
                config_row(label: "HTTP", value: "parallax.stynx.app:1080")
                config_row(label: "Username", value: "{carrier}-{username}")
                config_row(label: "Password", value: "your-api-key")
            </div>
        </div>
    }
}

#[component]
async fn config_row(label: &str, value: &str) -> Result {
    view! {
        <div class="flex justify-between items-center py-2 border-b border-zinc-800">
            <span class="text-zinc-400">(label)</span>
            <code class="text-emerald-400 text-xs bg-zinc-800 px-2 py-1 rounded">(value)</code>
        </div>
    }
}

#[component]
async fn carriers_card() -> Result {
    view! {
        <div class="rounded-lg border border-zinc-800 bg-zinc-900 p-6">
            <h2 class="text-lg font-semibold mb-4">"Carriers"</h2>
            <div class="space-y-3">
                carrier_row(name: "Telkomsel", country: "ID", online: true)
                carrier_row(name: "Indosat", country: "ID", online: true)
                carrier_row(name: "XL Axiata", country: "ID", online: false)
            </div>
        </div>
    }
}

#[component]
async fn carrier_row(name: &str, country: &str, online: bool) -> Result {
    let (dot, text) = if online {
        ("bg-emerald-500", "Online")
    } else {
        ("bg-zinc-600", "Offline")
    };
    view! {
        <div class="flex items-center justify-between py-2 border-b border-zinc-800">
            <div class="flex items-center gap-3">
                <span class="text-xs text-zinc-500 uppercase w-6">(country)</span>
                <span>(name)</span>
            </div>
            <div class="flex items-center gap-2">
                <span class=("w-2 h-2 rounded-full ".to_owned() + dot)></span>
                <span class="text-sm text-zinc-400">(text)</span>
            </div>
        </div>
    }
}
