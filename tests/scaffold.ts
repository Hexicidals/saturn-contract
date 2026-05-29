import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";

describe("pump-fees-to-jupiter-perps scaffold", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  it("loads the workspace program", () => {
    const program = anchor.workspace.PumpFeesToJupiterPerps;
    expect(program.programId.toBase58()).to.equal(
      "F3WS96pF3QCpovpw4hcEr1cvmroNYuEZqKYg9G8n9Sw1",
    );
  });
});

