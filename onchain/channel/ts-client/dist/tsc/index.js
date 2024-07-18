"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.executionClaimAddress = exports.deploymentAddress = exports.executionAddress = exports.BONSOL_PROGRAM_ID = void 0;
const tslib_1 = require("tslib");
const web3_js_1 = require("@solana/web3.js");
const sha3_1 = require("@noble/hashes/sha3");
tslib_1.__exportStar(require("./execute"), exports);
tslib_1.__exportStar(require("./deploy"), exports);
exports.BONSOL_PROGRAM_ID = "BoNsHRcyLLNdtnoDf8hiCNZpyehMC4FDMxs6NTxFi3ew";
async function executionAddress(requester, executionId) {
    const addressEncoder = (0, web3_js_1.getAddressEncoder)();
    const pd = await (0, web3_js_1.getProgramDerivedAddress)({
        programAddress: (0, web3_js_1.address)(exports.BONSOL_PROGRAM_ID),
        seeds: [
            Buffer.from("execution", "utf-8"),
            addressEncoder.encode(requester),
            Buffer.from(executionId, "utf-8")
        ]
    });
    return pd[0];
}
exports.executionAddress = executionAddress;
async function deploymentAddress(imageId) {
    const imageIdHash = (0, sha3_1.keccak_256)(Buffer.from(imageId, "utf-8"));
    const pd = await (0, web3_js_1.getProgramDerivedAddress)({
        programAddress: (0, web3_js_1.address)(exports.BONSOL_PROGRAM_ID),
        seeds: [
            Buffer.from("deployment", "utf-8"),
            imageIdHash
        ]
    });
    return pd[0];
}
exports.deploymentAddress = deploymentAddress;
async function executionClaimAddress(requester, executionId, claimer) {
    const addressEncoder = (0, web3_js_1.getAddressEncoder)();
    const pd = await (0, web3_js_1.getProgramDerivedAddress)({
        programAddress: (0, web3_js_1.address)(exports.BONSOL_PROGRAM_ID),
        seeds: [
            Buffer.from("execution-claim", "utf-8"),
            addressEncoder.encode(requester),
            Buffer.from(executionId, "utf-8"),
            addressEncoder.encode(claimer)
        ]
    });
    return pd[0];
}
exports.executionClaimAddress = executionClaimAddress;
//# sourceMappingURL=index.js.map