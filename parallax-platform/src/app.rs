mod dashboard;
mod proxies;
mod users;

use topcoat::{
    Result,
    context::{Cx, app_context},
    htmx::hx_request,
    router::{layout, request},
    tailwind,
    view::{component, view},
};

use crate::api::ApiClient;

#[layout("/")]
async fn root(cx: &Cx, slot: Result) -> Result {
    if hx_request(cx) {
        return slot;
    }

    let api = app_context::<ApiClient>(cx);
    let online = api.health().await;
    let (badge, badge_cls) = if online {
        ("Online", "text-xs px-2 py-1 rounded bg-emerald-900 text-emerald-400")
    } else {
        ("Offline", "text-xs px-2 py-1 rounded bg-red-900 text-red-400")
    };
    let path = request::uri(cx).path().to_string();

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>"Parallax - Indonesian Mobile Proxy Platform"</title>
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
                <script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.10/dist/htmx.min.js"></script>
                <script defer="" src="https://cdn.jsdelivr.net/npm/alpinejs@3.14.9/dist/cdn.min.js"></script>
                topcoat::dev::script()
            </head>
            <body class="bg-zinc-950 text-zinc-100 min-h-screen antialiased">
                <div class="flex min-h-screen">
                    sidebar(current_path: &path)
                    <main class="flex-1 ml-64">
                        topbar(badge: badge, badge_cls: badge_cls)
                        <div id="content" class="p-8">
                            (slot?)
                        </div>
                    </main>
                </div>
            </body>
        </html>
    }
}

#[component]
async fn sidebar(current_path: &str) -> Result {
    view! {
        <aside class="fixed inset-y-0 left-0 w-64 border-r border-zinc-800 bg-zinc-900 flex flex-col">
            <div class="h-16 px-6 flex flex-col justify-center border-b border-zinc-800">
                <h1 class="text-lg font-bold tracking-tight leading-tight">"Parallax"</h1>
                <p class="text-xs text-zinc-500">"Indonesian Mobile Proxy"</p>
            </div>
            <nav class="flex-1 px-4 py-4 space-y-1">
                nav_link(href: "/", label: "Dashboard", icon: "~", active: current_path == "/")
                nav_link(href: "/users", label: "Users", icon: "@", active: current_path.starts_with("/users"))
                nav_link(href: "/proxies", label: "Proxies", icon: "#", active: current_path.starts_with("/proxies"))
            </nav>
            <div class="px-4 py-4 border-t border-zinc-800" x-data="{ open: false }">
                <div
                    class="flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer hover:bg-zinc-800 transition-colors"
                    x-on:click="open = !open"
                >
                    <div class="w-8 h-8 rounded-full bg-zinc-700 flex items-center justify-center text-sm font-medium">"A"</div>
                    <div class="flex-1">
                        <p class="text-sm font-medium">"Admin"</p>
                        <p class="text-xs text-zinc-500">"admin@parallax.id"</p>
                    </div>
                    <svg class="w-4 h-4 text-zinc-500 transition-transform" x-bind:class="open && 'rotate-180'" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                    </svg>
                </div>
                <div x-show="open" x-transition="" class="mt-2 px-3 space-y-1">
                    <a href="#" class="block text-xs text-zinc-500 hover:text-zinc-300 py-1">"Settings"</a>
                    <a href="#" class="block text-xs text-zinc-500 hover:text-red-400 py-1">"Logout"</a>
                </div>
            </div>
        </aside>
    }
}

#[component]
async fn nav_link(href: &str, label: &str, icon: &str, active: bool) -> Result {
    let link_cls = if active {
        "flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors text-zinc-100 bg-zinc-800"
    } else {
        "flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800"
    };
    let icon_cls = if active {
        "w-5 text-center font-mono text-emerald-400"
    } else {
        "w-5 text-center font-mono text-zinc-500"
    };
    view! {
        <a href=(href) class=(link_cls)>
            <span class=(icon_cls)>(icon)</span>
            (label)
        </a>
    }
}

#[component]
async fn topbar(badge: &str, badge_cls: &str) -> Result {
    view! {
        <div class="h-16 border-b border-zinc-800 px-8 flex items-center justify-end gap-4">
            <span class=(badge_cls)>(badge)</span>
        </div>
    }
}
