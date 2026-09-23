mod dashboard;
mod proxies;
mod users;

use topcoat::{
    Result,
    router::layout,
    tailwind,
    view::{component, view},
};

#[layout("/")]
async fn root(slot: Result) -> Result {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>"Parallax - Indonesian Mobile Proxy Platform"</title>
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
                topcoat::dev::script()
            </head>
            <body class="bg-zinc-950 text-zinc-100 min-h-screen antialiased">
                <div class="flex min-h-screen">
                    sidebar()
                    <main class="flex-1 ml-64">
                        <div class="h-16 border-b border-zinc-800 px-8 flex items-center">
                            topbar()
                        </div>
                        <div class="p-8">
                            (slot?)
                        </div>
                    </main>
                </div>
            </body>
        </html>
    }
}

#[component]
async fn sidebar() -> Result {
    view! {
        <aside class="fixed inset-y-0 left-0 w-64 border-r border-zinc-800 bg-zinc-900 flex flex-col">
            <div class="h-16 px-6 flex flex-col justify-center border-b border-zinc-800">
                <h1 class="text-lg font-bold tracking-tight leading-tight">"Parallax"</h1>
                <p class="text-xs text-zinc-500">"Indonesian Mobile Proxy"</p>
            </div>
            <nav class="flex-1 px-4 py-4 space-y-1">
                nav_item(href: "/", label: "Dashboard", icon: "~")
                nav_item(href: "/users", label: "Users", icon: "@")
                nav_item(href: "/proxies", label: "Proxies", icon: "#")
            </nav>
            <div class="px-4 py-4 border-t border-zinc-800">
                <div class="flex items-center gap-3 px-3 py-2">
                    <div class="w-8 h-8 rounded-full bg-zinc-700 flex items-center justify-center text-sm font-medium">"A"</div>
                    <div>
                        <p class="text-sm font-medium">"Admin"</p>
                        <p class="text-xs text-zinc-500">"admin@parallax.id"</p>
                    </div>
                </div>
            </div>
        </aside>
    }
}

#[component]
async fn nav_item(href: &str, label: &str, icon: &str) -> Result {
    view! {
        <a
            href=(href)
            class="flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 transition-colors"
        >
            <span class="w-5 text-center font-mono text-zinc-500">(icon)</span>
            (label)
        </a>
    }
}

#[component]
async fn topbar() -> Result {
    view! {
        <div class="flex items-center justify-between w-full">
            <div></div>
            <div class="flex items-center gap-4">
                <span class="text-xs px-2 py-1 rounded bg-emerald-900 text-emerald-400" id="status-badge">"loading..."</span>
            </div>
        </div>
        <script>
            "fetch('/api/health').then(r => r.ok ? "
            "document.getElementById('status-badge').textContent = 'online' : null"
            ").catch(() => document.getElementById('status-badge').textContent = 'offline')"
        </script>
    }
}
