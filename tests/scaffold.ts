import assert from "node:assert/strict";
import { PROGRAM_ID } from "../client/constants";

describe("pump-fees-to-jupiter-perps scaffold", () => {
  it("exports the configured program id", () => {
    assert.equal(
      PROGRAM_ID.toBase58(),
      "FjDSgr7sF8o3rwqnSp9m87xjEX18XxgWELhNVxwVkjDz",
    );
  });
});
