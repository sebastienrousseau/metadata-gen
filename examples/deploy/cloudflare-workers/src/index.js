// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// A Cloudflare Worker that runs the metadata-gen WASI component
// (transpiled to core wasm + JS by jco) and answers POST requests with
// the document's front matter as JSON. See docs/deploy/cloudflare-workers.md.

import { instantiate } from "./gen/metadata-gen.js";
import { WASIShim } from "@bytecodealliance/preview2-shim/instantiation";
import core1 from "./gen/metadata-gen.core.wasm";
import core2 from "./gen/metadata-gen.core2.wasm";
import core3 from "./gen/metadata-gen.core3.wasm";

const modules = {
  "metadata-gen.core.wasm": core1,
  "metadata-gen.core2.wasm": core2,
  "metadata-gen.core3.wasm": core3,
};

let component;
async function load() {
  component ??= await instantiate(
    (path) => modules[path],
    new WASIShim().getImportObject(),
  );
  return component;
}

export default {
  async fetch(request) {
    if (request.method !== "POST") {
      return new Response("POST a Markdown document\n", { status: 405 });
    }
    const { metadata } = await load();
    try {
      const entries = metadata.extract(await request.text());
      return Response.json(Object.fromEntries(entries.map((e) => [e.key, e.value])));
    } catch (e) {
      return Response.json({ error: e.payload ?? String(e) }, { status: 422 });
    }
  },
};
