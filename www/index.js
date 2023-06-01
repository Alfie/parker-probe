import {ElfFile} from "solana-disassembler";
import { memory } from "solana-disassembler/solana_disassembler_bg"
import {Connection, PublicKey} from "@solana/web3.js"
var Buffer = require('buffer/').Buffer

//TODO: set up process.env so we don't expose out api key
const connection = new Connection("https://rpc.helius.xyz/?api-key=56de2bc4-02f3-492b-b608-8b970b885691")
const canvas = document.getElementById("workbench");
const ctx = canvas.getContext("2d");

//TODO: store blocks of assembly insns in arrays in the rust program
//      use a for loop to iterate over the insn array and fillText adding approximately 24 px to the y val each element
//      calculate insn w/ most chars and set width of stokeRect according to that value.
//      make rectangles including assembly blocks moveable
const draw = () => {
    ctx.strokeRect(50, 50, 75, 24);

    ctx.font = "12px sans-serif";
    ctx.fillText("hello, world!", 55, 65);
}

async function test() {
    //Pull binary data from the blockchain.
    const pk = new PublicKey("A3zRq3PDXfFNawQnVGicAf2GDUEWppXo6EVk3gmZ2Ucu");
    const info = await connection.getAccountInfo(pk);

    const elfFile = ElfFile.new();
    elfFile.load(info.data);
    elfFile.list_sections();
    elfFile.disassemble();
}
draw()
test()