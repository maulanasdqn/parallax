use serde::Deserialize;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{content::Form, error::{SeeOther, see_other}, page, route},
    view::{component, view},
};
use uuid::Uuid;

use crate::db::Database;

#[page("/users")]
async fn users_page(cx: &Cx) -> Result {
    let db = app_context::<Database>(cx);
    let users = db.list_users().await;
    let plans = db.list_plans().await;

    view! {
        <div>
            <div class="flex items-center justify-between">
                <div>
                    <h2 class="text-2xl font-semibold tracking-tight">"Users"</h2>
                    <p class="text-zinc-400 mt-1 text-sm">"Manage proxy users and API keys"</p>
                </div>
            </div>

            <form method="POST" action="/users/create" class="mt-6 rounded-lg border border-zinc-800 bg-zinc-900 p-5">
                <h3 class="text-sm font-semibold mb-4">"Create User"</h3>
                <div class="flex gap-4 items-end">
                    <div class="flex-1">
                        <label class="block text-xs text-zinc-500 mb-1">"Email"</label>
                        <input
                            name="email"
                            type="email"
                            required="true"
                            placeholder="user@example.com"
                            class="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-sm focus:border-emerald-500 focus:outline-none"
                        >
                    </div>
                    <div class="w-48">
                        <label class="block text-xs text-zinc-500 mb-1">"Plan"</label>
                        <select name="plan_id" class="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-sm focus:border-emerald-500 focus:outline-none">
                            for plan in &plans {
                                <option value=(plan.id.to_string())>(plan.name.as_str())</option>
                            }
                        </select>
                    </div>
                    <button
                        type="submit"
                        class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-sm font-medium rounded-lg transition-colors"
                    >
                        "Create"
                    </button>
                </div>
            </form>

            <div class="mt-6 rounded-lg border border-zinc-800 bg-zinc-900 overflow-hidden">
                <table class="w-full text-sm">
                    <thead>
                        <tr class="border-b border-zinc-800 text-left text-xs text-zinc-500 uppercase tracking-wider">
                            <th class="px-5 py-3">"Email"</th>
                            <th class="px-5 py-3">"Plan"</th>
                            <th class="px-5 py-3">"API Key"</th>
                            <th class="px-5 py-3">"Status"</th>
                            <th class="px-5 py-3">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-zinc-800">
                        for user in &users {
                            let plan_name = plans.iter()
                                .find(|p| p.id == user.plan_id)
                                .map(|p| p.name.as_str())
                                .unwrap_or("Unknown");
                            let (badge_cls, badge_txt) = if user.active {
                                ("bg-emerald-900 text-emerald-400", "Active")
                            } else {
                                ("bg-red-900 text-red-400", "Suspended")
                            };
                            <tr class="hover:bg-zinc-800/50 transition-colors">
                                <td class="px-5 py-3 font-medium">(user.email.as_str())</td>
                                <td class="px-5 py-3 text-zinc-400">(plan_name)</td>
                                <td class="px-5 py-3">
                                    <code class="text-xs bg-zinc-800 px-2 py-1 rounded text-zinc-300">(user.api_key.as_str())</code>
                                </td>
                                <td class="px-5 py-3">
                                    <span class=("text-xs px-2 py-1 rounded ".to_owned() + badge_cls)>(badge_txt)</span>
                                </td>
                                <td class="px-5 py-3">
                                    <div class="flex gap-3">
                                        <form method="POST" action="/users/action">
                                            <input type="hidden" name="id" value=(user.id.to_string())>
                                            <input type="hidden" name="action" value="regenerate">
                                            <button class="text-xs text-zinc-500 hover:text-emerald-400">"Regen Key"</button>
                                        </form>
                                        <form method="POST" action="/users/action">
                                            <input type="hidden" name="id" value=(user.id.to_string())>
                                            <input type="hidden" name="action" value="toggle">
                                            <button class="text-xs text-zinc-500 hover:text-yellow-400">
                                                if user.active { "Suspend" } else { "Activate" }
                                            </button>
                                        </form>
                                        <form method="POST" action="/users/action">
                                            <input type="hidden" name="id" value=(user.id.to_string())>
                                            <input type="hidden" name="action" value="delete">
                                            <button class="text-xs text-zinc-500 hover:text-red-400">"Delete"</button>
                                        </form>
                                    </div>
                                </td>
                            </tr>
                        }
                    </tbody>
                </table>
                if users.is_empty() {
                    <div class="px-5 py-8 text-center text-zinc-500 text-sm">"No users yet"</div>
                }
            </div>

            <div class="mt-6 grid grid-cols-1 md:grid-cols-3 gap-4">
                for plan in &plans {
                    let price = format!("{}k", plan.price_cents / 100);
                    let bw = format!("{} GB", plan.bandwidth_limit_bytes / (1024 * 1024 * 1024));
                    let sessions = plan.concurrent_limit.to_string();
                    plan_card(name: plan.name.as_str(), price: &price, bw: &bw, sessions: &sessions)
                }
            </div>
        </div>
    }
}

#[derive(Deserialize)]
struct CreateUserForm {
    email: String,
    plan_id: String,
}

#[route(POST "/users/create")]
async fn create_user(cx: &Cx, Form(form): Form<CreateUserForm>) -> Result<SeeOther> {
    let db = app_context::<Database>(cx);
    let plan_id: Uuid = form.plan_id.parse().unwrap_or_default();
    db.create_user(form.email, plan_id).await;
    Ok(see_other("/users"))
}

#[derive(Deserialize)]
struct UserActionForm {
    id: String,
    action: String,
}

#[route(POST "/users/action")]
async fn user_action(cx: &Cx, Form(form): Form<UserActionForm>) -> Result<SeeOther> {
    let db = app_context::<Database>(cx);
    let id: Uuid = form.id.parse().unwrap_or_default();
    match form.action.as_str() {
        "toggle" => db.toggle_user(id).await,
        "delete" => db.delete_user(id).await,
        "regenerate" => db.regenerate_key(id).await,
        _ => {}
    }
    Ok(see_other("/users"))
}

#[component]
async fn plan_card(name: &str, price: &str, bw: &str, sessions: &str) -> Result {
    view! {
        <div class="rounded-lg border border-zinc-800 bg-zinc-900 p-5">
            <h3 class="font-semibold">(name)</h3>
            <p class="text-2xl font-bold mt-2">"Rp "(price)<span class="text-sm font-normal text-zinc-500">"/mo"</span></p>
            <div class="mt-4 space-y-2 text-sm text-zinc-400">
                <p>(bw)" bandwidth"</p>
                <p>(sessions)" concurrent sessions"</p>
                <p>"All endpoints"</p>
            </div>
        </div>
    }
}
