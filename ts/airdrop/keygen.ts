import { createKeyPairSignerFromBytes } from "@solana/kit";
import bs58 from 'bs58';
import promptSync from 'prompt-sync';

async function generateKeypair() {
    const keypair = await crypto.subtle.generateKey(
        {
            name: "Ed25519",
        },
        true,
        ["sign", "verify"]
    );

    const privateKeyJwk = await crypto.subtle.exportKey('jwk', keypair.privateKey);
    const privateKeyBase64 = privateKeyJwk.d;

    if (!privateKeyBase64) throw new Error('Failed to get private key bytes');

    const privateKeyBytes = new Uint8Array(Buffer.from(privateKeyBase64, 'base64'));

    const publicKeyBytes = new Uint8Array(await crypto.subtle.exportKey('raw', keypair.publicKey));

    const keypairBytes = new Uint8Array([...privateKeyBytes, ...publicKeyBytes]);

    const signer = await createKeyPairSignerFromBytes(keypairBytes);

    console.log(`You have generated a new Solana wallet: ${signer.address}`);

    console.log(`To save your wallet, copy and paste your keypair bytes into a JSON file:`);
    console.log(`[${Array.from(keypairBytes).join(',')}]`);

    return signer;
}

function base58ToWallet(base58?: string) {
    const prompt = promptSync();

    if (!base58) {
        console.log("Enter your base58 encoded private key:");
        base58 = prompt('> ').trim();
    }

    try {
        const wallet = bs58.decode(base58);
        console.log("Wallet bytes:", Array.from(wallet));
        return Array.from(wallet);
    } catch (error) {
        console.error("Error decoding base58:", error);
        return null;
    }
}

function walletToBase58() {
    const prompt = promptSync();
    console.log("Enter your wallet bytes (comma-separated):");
    const input = prompt('> ').trim();

    try {
        let walletBytes: number[];

        if (input.startsWith('[') && input.endsWith(']')) {
            walletBytes = input.slice(1, -1).split(',').map((s: string) => parseInt(s.trim()));
        } else {
            walletBytes = input.split(/[\s,]+/).map((s: string) => parseInt(s.trim()));
        }

        if (walletBytes.some(isNaN)) {
            throw new Error("Invalid number format");
        }

        const walletArray = new Uint8Array(walletBytes);
        const base58 = bs58.encode(walletArray);
        console.log("Base58 encoded:", base58);
        return base58;
    } catch (error) {
        console.error("Error encoding to base58:", error);
        return null;
    }
}

export { base58ToWallet, walletToBase58 };

generateKeypair().catch(console.error);