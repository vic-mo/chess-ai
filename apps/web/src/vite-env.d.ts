/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_ENGINE_MODE?: string;
  readonly VITE_ENGINE_SERVER_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
