import { Program } from "@coral-xyz/anchor";
import { Connection, PublicKey } from "@solana/web3.js";
import * as fs from "fs";
import * as path from "path";

async function fetchAndSaveIDL() {
  const PROGRAM_ID = "TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM";
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");

  try {
    console.log(`Fetching IDL for program: ${PROGRAM_ID}...`);

    const programId = new PublicKey(PROGRAM_ID);
    const idl = await Program.fetchIdl(programId, { connection });

    if (!idl) {
      console.error("IDL not found for this program");
      return;
    }

    console.log("IDL fetched successfully!");

    const programsDir = path.join(process.cwd(), "programs");
    if (!fs.existsSync(programsDir)) {
      fs.mkdirSync(programsDir, { recursive: true });
    }

    const idlPath = path.join(programsDir, "Turbin3_prereq.json");
    fs.writeFileSync(idlPath, JSON.stringify(idl, null, 2));

    console.log(`IDL saved to: ${idlPath}`);
    console.log("\nIDL Preview:");
    console.log(JSON.stringify(idl, null, 2));

  } catch (error) {
    console.error("Error fetching IDL:", error);
    console.log("\nTrying alternative method using anchor CLI...");
    console.log(`Run: anchor idl fetch ${PROGRAM_ID} --provider.cluster devnet`);
  }
}

fetchAndSaveIDL();