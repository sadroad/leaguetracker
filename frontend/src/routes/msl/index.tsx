import { component$ } from "@builder.io/qwik";

export default component$(() => {
  return (
    <div class="bg-gray-200">
      <header class="text-center py-16 bg-blue-600 text-white">
        <h1 class="text-4xl">Welcome to Metashift League</h1>
        <p class="mt-4 text-xl">
          Your premier destination for amateur League of Legends competitions.
        </p>
      </header>

      <main class="px-4 py-8">
        <section class="mb-8">
          <h2 class="text-2xl mb-4">About the League</h2>
        </section>

        <section class="mb-8">
          <h2 class="text-2xl mb-4">Rules</h2>
        </section>

        <section>
          <h2 class="text-2xl mb-4">Upcoming Events</h2>
        </section>

        <section class="mt-16 text-center">
          <button class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
            Join Now!
          </button>
        </section>
        <section class="mb-8">
          <h2 class="text-2xl mb-4">Teams</h2>
          <table class="table-fixed">
            <thead>
              <tr>
                <th class="w-1/2 px-4 py-2">Team Name</th>
                <th class="w-1/4 px-4 py-2">Division</th>
                <th class="w-1/4 px-4 py-2">Players</th>
              </tr>
            </thead>
            <tbody></tbody>
          </table>
        </section>
        <section class="mb-8">
          <h2 class="text-2xl mb-4">Login</h2>
          <form action="/login" method="post">
            <label class="block">
              <span class="text-gray-700">Username</span>
              <input
                class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50"
                type="text"
                placeholder="Username"
              />
            </label>
            <label class="block mt-4">
              <span class="text-gray-700">Password</span>
              <input
                class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50"
                type="password"
                placeholder="Password"
              />
            </label>
            <button
              class="mt-4 bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
              type="submit"
            >
              Login
            </button>
          </form>
        </section>

        <section class="mb-8">
          <h2 class="text-2xl mb-4">Manage Roster</h2>
        </section>
      </main>
    </div>
  );
});
