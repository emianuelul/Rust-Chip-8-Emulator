import { serve } from "bun";
import index from "./index.html";

const server = serve({
  routes: {
    "/rust_test_bg.wasm": Bun.file("./rust-lib/pkg/rust_test_bg.wasm"),

    "/chip-8-tests/*": Bun.file("./chip-8_test"),

    "/*": index,
  },

  development: process.env.NODE_ENV !== "production" && {
    hmr: true,

    console: true,
  },
});

console.log(`🚀 Server running at ${server.url}`);
