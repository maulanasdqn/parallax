use topcoat::{
    Result,
    router::page,
    view::view,
};

#[page("/users")]
async fn users_page() -> Result {
    view! {
        <div>
            <div class="flex items-center justify-between">
                <div>
                    <h2 class="text-2xl font-semibold tracking-tight">"Users"</h2>
                    <p class="text-zinc-400 mt-1 text-sm">"Manage proxy users and API keys"</p>
                </div>
                <button
                    id="btn-create"
                    class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-sm font-medium rounded-lg transition-colors"
                >
                    "+ Add User"
                </button>
            </div>

            <div id="create-form" class="mt-6 rounded-lg border border-zinc-800 bg-zinc-900 p-5 hidden">
                <h3 class="text-sm font-semibold mb-4">"Create User"</h3>
                <div class="flex gap-4 items-end">
                    <div class="flex-1">
                        <label class="block text-xs text-zinc-500 mb-1">"Email"</label>
                        <input
                            id="input-email"
                            type="email"
                            placeholder="user@example.com"
                            class="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-sm focus:border-emerald-500 focus:outline-none"
                        >
                    </div>
                    <div class="w-48">
                        <label class="block text-xs text-zinc-500 mb-1">"Plan"</label>
                        <select id="input-plan" class="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-sm focus:border-emerald-500 focus:outline-none">
                        </select>
                    </div>
                    <button
                        id="btn-submit"
                        class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-sm font-medium rounded-lg transition-colors"
                    >
                        "Create"
                    </button>
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
                    <tbody id="users-table" class="divide-y divide-zinc-800">
                        <tr><td class="px-5 py-4 text-zinc-500" colspan="4">"Loading..."</td></tr>
                    </tbody>
                </table>
            </div>

            <div id="plans-grid" class="mt-6 grid grid-cols-1 md:grid-cols-3 gap-4"></div>
        </div>

        <script>
            "const API = window.__API_URL || 'http://localhost:3001';"
            "const table = document.getElementById('users-table');"
            "const form = document.getElementById('create-form');"
            "document.getElementById('btn-create').onclick = () => form.classList.toggle('hidden');"

            "function loadUsers() {"
            "  fetch(API + '/api/users').then(r => r.json()).then(users => {"
            "    if (!users.length) { table.innerHTML = '<tr><td class=\"px-5 py-8 text-center text-zinc-500\" colspan=\"4\">No users yet</td></tr>'; return; }"
            "    table.innerHTML = users.map(u => `<tr class=\"hover:bg-zinc-800/50\">"
            "      <td class=\"px-5 py-3 font-medium\">${u.email}</td>"
            "      <td class=\"px-5 py-3\"><code class=\"text-xs bg-zinc-800 px-2 py-1 rounded text-zinc-300\">${u.api_key}</code></td>"
            "      <td class=\"px-5 py-3\"><span class=\"text-xs px-2 py-1 rounded ${u.active ? 'bg-emerald-900 text-emerald-400' : 'bg-red-900 text-red-400'}\">${u.active ? 'Active' : 'Suspended'}</span></td>"
            "      <td class=\"px-5 py-3\"><div class=\"flex gap-3\">"
            "        <button onclick=\"regen('${u.id}')\" class=\"text-xs text-zinc-500 hover:text-emerald-400\">Regen Key</button>"
            "        <button onclick=\"toggle('${u.id}')\" class=\"text-xs text-zinc-500 hover:text-yellow-400\">${u.active ? 'Suspend' : 'Activate'}</button>"
            "        <button onclick=\"del('${u.id}')\" class=\"text-xs text-zinc-500 hover:text-red-400\">Delete</button>"
            "      </div></td></tr>`).join('');"
            "  });"
            "}"

            "document.getElementById('btn-submit').onclick = () => {"
            "  const email = document.getElementById('input-email').value;"
            "  const plan_id = document.getElementById('input-plan').value;"
            "  fetch(API + '/api/users', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify({email, plan_id}) })"
            "    .then(() => { loadUsers(); form.classList.add('hidden'); document.getElementById('input-email').value = ''; });"
            "};"

            "function toggle(id) { fetch(API + '/api/users/' + id + '/toggle', {method:'PATCH'}).then(loadUsers); }"
            "function del(id) { fetch(API + '/api/users/' + id, {method:'DELETE'}).then(loadUsers); }"
            "function regen(id) { fetch(API + '/api/users/' + id + '/regenerate', {method:'POST'}).then(loadUsers); }"

            "loadUsers();"
        </script>
    }
}
