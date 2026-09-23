use topcoat::{
    Result,
    context::{Cx, app_context},
    router::page,
    view::{component, view},
};

use crate::api::ApiClient;

#[page("/users")]
async fn users_page(cx: &Cx) -> Result {
    let api = app_context::<ApiClient>(cx);
    let users = api.list_users().await;

    view! {
        <div x-data="{ showForm: false }">
            <div class="flex items-center justify-between">
                <div>
                    <h2 class="text-2xl font-semibold tracking-tight">"Users"</h2>
                    <p class="text-zinc-400 mt-1 text-sm">"Manage proxy users and API keys"</p>
                </div>
                <button
                    x-on:click="showForm = !showForm"
                    class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-sm font-medium rounded-lg transition-colors"
                >
                    <span x-text="showForm ? 'Cancel' : '+ Add User'"></span>
                </button>
            </div>

            <div x-show="showForm" x-transition="" class="mt-6 rounded-lg border border-zinc-800 bg-zinc-900 p-5">
                <h3 class="text-sm font-semibold mb-4">"Create User"</h3>
                <div class="flex gap-4 items-end" x-data="{ email: '', plan_id: '' }">
                    <div class="flex-1">
                        <label class="block text-xs text-zinc-500 mb-1">"Email"</label>
                        <input
                            x-model="email"
                            type="email"
                            placeholder="user@example.com"
                            class="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-sm focus:border-emerald-500 focus:outline-none"
                        >
                    </div>
                    <div class="w-48">
                        <label class="block text-xs text-zinc-500 mb-1">"Plan"</label>
                        <select x-model="plan_id" class="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-sm focus:border-emerald-500 focus:outline-none">
                            <option value="">"Select plan"</option>
                        </select>
                    </div>
                    <button
                        hx-post="http://localhost:3001/api/users"
                        hx-swap="none"
                        hx-on-htmx-after-request="htmx.ajax('GET', '/users', {target:'#content', swap:'innerHTML'})"
                        x-on:click="$el.setAttribute('hx-vals', JSON.stringify({email, plan_id}))"
                        class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-sm font-medium rounded-lg transition-colors"
                    >"Create"</button>
                </div>
            </div>

            <div class="mt-6 rounded-lg border border-zinc-800 bg-zinc-900 overflow-hidden">
                <table class="w-full text-sm">
                    <thead>
                        <tr class="border-b border-zinc-800 text-left text-xs text-zinc-500 uppercase tracking-wider">
                            <th class="px-5 py-3">"Email"</th>
                            <th class="px-5 py-3">"API Key"</th>
                            <th class="px-5 py-3">"Status"</th>
                            <th class="px-5 py-3">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-zinc-800">
                        if users.is_empty() {
                            <tr>
                                <td class="px-5 py-8 text-center text-zinc-500" colspan="4">"No users yet"</td>
                            </tr>
                        }
                        for user in &users {
                            let (badge_cls, badge_txt) = if user.active {
                                ("bg-emerald-900 text-emerald-400", "Active")
                            } else {
                                ("bg-red-900 text-red-400", "Suspended")
                            };
                            let toggle_label = if user.active { "Suspend" } else { "Activate" };
                            let toggle_url = format!("http://localhost:3001/api/users/{}/toggle", user.id);
                            let delete_url = format!("http://localhost:3001/api/users/{}", user.id);
                            let regen_url = format!("http://localhost:3001/api/users/{}/regenerate", user.id);
                            <tr class="hover:bg-zinc-800/50 transition-colors" x-data="{ copied: false }">
                                <td class="px-5 py-3 font-medium">(user.email.as_str())</td>
                                <td class="px-5 py-3">
                                    <div class="flex items-center gap-2">
                                        <code class="text-xs bg-zinc-800 px-2 py-1 rounded text-zinc-300">(user.api_key.as_str())</code>
                                        <button
                                            x-on:click=(&format!("navigator.clipboard.writeText('{}'); copied = true; setTimeout(() => copied = false, 2000)", user.api_key))
                                            class="text-xs text-zinc-600 hover:text-zinc-400"
                                        >
                                            <span x-show="!copied">"copy"</span>
                                            <span x-show="copied" x-cloak="" class="text-emerald-400">"copied"</span>
                                        </button>
                                    </div>
                                </td>
                                <td class="px-5 py-3">
                                    <span class=("text-xs px-2 py-1 rounded ".to_owned() + badge_cls)>(badge_txt)</span>
                                </td>
                                <td class="px-5 py-3">
                                    <div class="flex gap-3">
                                        <button
                                            hx-post=(&regen_url)
                                            hx-swap="none"
                                            hx-on-htmx-after-request="htmx.ajax('GET', '/users', {target:'#content', swap:'innerHTML'})"
                                            class="text-xs text-zinc-500 hover:text-emerald-400"
                                        >"Regen Key"</button>
                                        <button
                                            hx-patch=(&toggle_url)
                                            hx-swap="none"
                                            hx-on-htmx-after-request="htmx.ajax('GET', '/users', {target:'#content', swap:'innerHTML'})"
                                            class="text-xs text-zinc-500 hover:text-yellow-400"
                                        >(toggle_label)</button>
                                        <button
                                            x-data="{ confirm: false }"
                                            x-on:click="if(confirm) { $el.dispatchEvent(new Event('confirmed')); confirm = false; } else { confirm = true; setTimeout(() => confirm = false, 3000); }"
                                            hx-delete=(&delete_url)
                                            hx-swap="none"
                                            hx-trigger="confirmed"
                                            hx-on-htmx-after-request="htmx.ajax('GET', '/users', {target:'#content', swap:'innerHTML'})"
                                            class="text-xs text-zinc-500 hover:text-red-400"
                                        >
                                            <span x-show="!confirm">"Delete"</span>
                                            <span x-show="confirm" class="text-red-400">"Confirm?"</span>
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        }
                    </tbody>
                </table>
            </div>
        </div>
    }
}
