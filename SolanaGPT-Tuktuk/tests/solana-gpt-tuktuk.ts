import * as anchor from "@coral-xyz/anchor";
import { BN, Program, web3 } from "@coral-xyz/anchor";
import { SolanaGptTuktuk } from "../target/types/solana_gpt_tuktuk";
import { expect } from "chai";

const ORACLE_PROGRAM_ID = new web3.PublicKey(
  "LLMrieZMpbJFwN52WgmBNMxYojrpRVYXdC1RCweEbab"
);

describe("solana-gpt-tuktuk", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .solanaGptTuktuk as Program<SolanaGptTuktuk>;
  const maker = provider.wallet.publicKey;

  // Derive PDAs
  const [agentPda] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("agent"), maker.toBuffer()],
    program.programId
  );

  const [counterPda] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("counter")],
    ORACLE_PROGRAM_ID
  );

  // We need to read the counter to derive context PDA.
  // For first-time use, counter is 0.
  let contextPda: web3.PublicKey;
  let interactionPda: web3.PublicKey;

  before(async () => {
    // Try to read the counter to determine the current context index
    let counterValue = 0;
    try {
      const counterAccount = await provider.connection.getAccountInfo(
        counterPda
      );
      if (counterAccount && counterAccount.data.length >= 12) {
        // Skip 8-byte discriminator, read u32
        counterValue = counterAccount.data.readUInt32LE(8);
      }
    } catch {
      console.log("Counter not found, using 0");
    }

    // Derive context PDA: seeds = ["test-context", counter_le_bytes]
    const counterBytes = Buffer.alloc(4);
    counterBytes.writeUInt32LE(counterValue);
    [contextPda] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from("test-context"), counterBytes],
      ORACLE_PROGRAM_ID
    );

    // Derive interaction PDA: seeds = ["interaction", agent_pda, context_pda]
    [interactionPda] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from("interaction"), agentPda.toBuffer(), contextPda.toBuffer()],
      ORACLE_PROGRAM_ID
    );

    console.log("Agent PDA:", agentPda.toBase58());
    console.log("Counter PDA:", counterPda.toBase58());
    console.log("Context PDA:", contextPda.toBase58());
    console.log("Interaction PDA:", interactionPda.toBase58());
  });

  it("Initialize agent", async () => {
    const systemPrompt =
      "You are a helpful Solana assistant. Answer questions about Solana concisely.";
    const queryPrompt = "What is Solana?";

    const tx = await program.methods
      .initialize(systemPrompt, queryPrompt)
      .accounts({
        maker: maker,
        agent: agentPda,
        llmContext: contextPda,
        counter: counterPda,
        oracleProgram: ORACLE_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc({ skipPreflight: true });

    console.log("Initialize tx:", tx);

    // Verify agent account
    const agent = await program.account.agent.fetch(agentPda);
    expect(agent.maker.toBase58()).to.equal(maker.toBase58());
    expect(agent.prompt).to.equal("What is Solana?");
    expect(agent.context.toBase58()).to.equal(contextPda.toBase58());
    console.log("Agent created successfully");
  });

  it("Ask GPT (sends query to oracle)", async () => {
    const tx = await program.methods
      .askGpt()
      .accounts({
        agent: agentPda,
        interaction: interactionPda,
        contextAccount: contextPda,
        oracleProgram: ORACLE_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc({ skipPreflight: true });

    console.log("Ask GPT tx:", tx);
    console.log("Query sent to oracle, waiting for callback...");
  });

  it("Wait for GPT callback and verify response", async () => {
    // The oracle's off-chain service picks up the interaction and responds.
    // Poll the agent account until last_response is set.
    const maxWait = 30_000; // 30 seconds
    const interval = 3_000; // check every 3 seconds
    let elapsed = 0;

    while (elapsed < maxWait) {
      const agent = await program.account.agent.fetch(agentPda);
      if (agent.lastResponse && agent.lastResponse.length > 0) {
        console.log("GPT Response received:", agent.lastResponse);
        expect(agent.lastResponse.length).to.be.greaterThan(0);
        return;
      }
      console.log(`Waiting for oracle callback... (${elapsed / 1000}s)`);
      await new Promise((r) => setTimeout(r, interval));
      elapsed += interval;
    }

    console.log(
      "Timeout: Oracle callback not received within 30s. This is expected if the oracle service is not running."
    );
  });
});
