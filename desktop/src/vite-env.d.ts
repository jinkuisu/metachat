/// <reference types="vite/client" />

declare module "*.svelte" {
  const component: any;
  export default component;
}

declare module "*.svelte.ts" {
  const content: any;
  export default content;
}
