import { appContext } from "./app-context.js";

/**
 * Implementation of Hirschberg's algorithm
 */
export class Scorer {
  threshold;
  cache;
  arr;
  constructor(threshold) {
    this.threshold = threshold;
    this.cache = new Uint32Array(0x10000);
    this.arr = new Uint32Array(0x10000);
  }
  async test(candidate) {
    if (!appContext.get("titles").length) {
      await fetch("/titles")
        .then((res) => res.json())
        .then((titles) => {
          appContext.set(
            "titles",
            titles.map((t) => t.toLowerCase())
          );
        });
    }
    const potentialMatches = appContext.get("titles");
    const top = [];
    if (candidate.length === 0) {
      return;
    }
    let i = -1;
    const start = performance.now();
    while (++i < potentialMatches.length) {
      let y = potentialMatches[i];
      if (candidate === y) {
        top.push({ value: y, score: 0 });
      }
      // Sort the strings so that they are in length order
      let a = candidate;
      let b = y.slice(0, candidate.length);
      let aLen = a.length;
      let bLen = b.length;
      // We can trim off shared suffixes
      // Note: `~-` is the bitwise way to perform a `- 1` operation
      while (aLen > 0 && a.charCodeAt(~-aLen) === b.charCodeAt(~-bLen)) {
        aLen--;
        bLen--;
      }
      // Now trim off any shared prefixes
      let prefixIdx = 0;
      while (
        prefixIdx < aLen &&
        a.charCodeAt(prefixIdx) === b.charCodeAt(prefixIdx)
      ) {
        prefixIdx++;
      }
      aLen -= prefixIdx;
      bLen -= prefixIdx;

      if (aLen === 0) {
        if (bLen < this.threshold) {
          top.push({ value: y, score: bLen });
        }
      }
      let result;
      let idx = -1;
      while (++idx < aLen) {
        this.cache[idx] = a.charCodeAt(prefixIdx + idx);
        this.arr[idx] = idx + 1;
      }
      let bdx = -1;
      let charCode, register0, register1;
      while (++bdx < bLen) {
        charCode = b.charCodeAt(prefixIdx + bdx);
        register0 = bdx + 1;
        result = bdx;
        let cursor = -1;
        while (++cursor < aLen) {
          register1 =
            charCode === this.cache[cursor] ? register0 : register0 + 1;
          register0 = this.arr[cursor];
          result = this.arr[cursor] =
            register0 > result
              ? register1 > result
                ? result + 1
                : register1
              : register1 > register0
              ? register0 + 1
              : register1;
        }
      }
      if (result < this.threshold) {
        top.push({ value: y, score: result });
      }
    }
    const end = performance.now();
    console.log(`Searching took: ${end - start}ms`);
    return top;
  }
}
