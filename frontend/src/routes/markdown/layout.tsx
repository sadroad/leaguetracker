import { component$, Slot } from "@builder.io/qwik";

export default component$(() => {
  return (
    <>
      <article class="prose prose-a:text-blue-600 hover:prose-a:text-blue-500 prose-code:before:hidden prose-code:after:hidden">
        <Slot />
      </article>
    </>
  );
});
