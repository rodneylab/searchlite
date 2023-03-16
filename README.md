<img src="./images/rodneylab-github-searchlite.png" alt="Rodney Lab search lite down Github banner">

<p align="center" style="display:grid;place-items:center;margin-block:2rem">
  <a aria-label="Open Rodney Lab site" href="https://rodneylab.com" rel="nofollow noopener noreferrer">
    <img alt="Rodney Lab logo" src="https://rodneylab.com/assets/icon.png" width="60" />
  </a>
</p>
<h1 align="center">
  searchlite
</h1>

Rust WASM tool to manipulate HTML searching for an input search term. Searchlite will wrap matching text elements in a pair of HTML `<mark>` tags. Browsers will highlight these matches by default.

Module can be used within a web app or in serverless middleware to highlight search terms in an HTML response.

Uses `html5ever` and `aho-corasick` Rust crates under the hood.

## Compile WASM

1. Clone the project and change into the project directory. Then run these
   commands:

```shell
cargo install wasm-pack # skip if you already have it installed
wasm-pack build --target web
```

2. Copy the generated `pkg` folder into your JavaScript or TypeScript project.
3. Import and use the code in one of your project source files (expected output
   is as shown in previous section):

- Generate highlighted HTML

`highlight_search_terms` takes two arguments: the input HTML and the search term. Separate multiple search terms with a space (e.g. `"apple pear"`).

```typescript
import init, { highlight_search_terms as highlight } from "pkg/searchlite.js";

await init();

// alternative if top level await is not available
(async () => {
  await init();
})();

const highlightedHtml = highlight(
  "<h2>Heading</h2><p>Nobody likes maple in their apple flavoured Snapple. APPLE</p>",
  "apple",
);
```

Output (`highlightedHTML`):

```html
<h2>Heading</h2>
<p>
  Nobody likes maple in their <mark id="search-match">apple</mark> flavoured
  Sn<mark>apple</mark>. <mark>APPLE</mark>
</p>
```

Note the `id` added to the first search match. You can use this to scroll the first match into view.

<img src="./images/searchlite-example.png" alt="Searchlite example screen capture shows all instances of the letters `apple` highlighted whether in the work apple (lower case) alone, within the word Snapple or APPLE (upper case)">

## 🗺️ Roadmap

No firm course laid in.

- Possibly add stemmer, though this might be better handled externally. A stemmer will match on related words; film and filming for an input of films as an example. The [Porter Algorithm](https://tartarus.org/martin/PorterStemmer/def.txt) is often used for this.
- Possibly add a utility function to generate a match snippet for use in result pages that show matches across various documents.

## ☎️ Reach Out

Feel free to jump into the
[Rodney Lab matrix chat room](https://matrix.to/#/%23rodney:matrix.org).
