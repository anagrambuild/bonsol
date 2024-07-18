"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Deploy = void 0;
const tslib_1 = require("tslib");
const web3_js_1 = require("@solana/web3.js");
const bonsol_schemas_1 = require("bonsol-schemas");
const flatbuffers = tslib_1.__importStar(require("flatbuffers"));
const _1 = require(".");
async function Deploy(params) {
    const { imageUrl, imageId, programName, deployer, payer, inputs, imageSize } = params;
    //eunsure valid url
    if (!URL.canParse(imageUrl)) {
        throw new Error("Invalid URL");
    }
    const builder = new flatbuffers.Builder(0);
    const imgId = builder.createString(imageId);
    const imgName = builder.createString(programName);
    const imgURL = builder.createString(imageUrl);
    const ownerBytes = (0, web3_js_1.getAddressEncoder)().encode(deployer);
    //@ts-ignore
    const ownerBuf = bonsol_schemas_1.DeployV1.createOwnerVector(builder, ownerBytes);
    let iv = bonsol_schemas_1.DeployV1.createInputsVector(builder, inputs);
    bonsol_schemas_1.DeployV1.startDeployV1(builder);
    bonsol_schemas_1.DeployV1.addUrl(builder, imgURL);
    bonsol_schemas_1.DeployV1.addSize(builder, imageSize);
    bonsol_schemas_1.DeployV1.addImageId(builder, imgId);
    bonsol_schemas_1.DeployV1.addProgramName(builder, imgName);
    bonsol_schemas_1.DeployV1.addInputs(builder, iv);
    bonsol_schemas_1.DeployV1.addOwner(builder, ownerBuf);
    const deploy = bonsol_schemas_1.DeployV1.endDeployV1(builder);
    builder.finish(deploy);
    const deployData = builder.asUint8Array();
    const erv = bonsol_schemas_1.ChannelInstruction.createDeployV1Vector(builder, deployData);
    bonsol_schemas_1.ChannelInstruction.startChannelInstruction(builder);
    bonsol_schemas_1.ChannelInstruction.addDeployV1(builder, erv);
    bonsol_schemas_1.ChannelInstruction.addIxType(builder, bonsol_schemas_1.ChannelInstructionIxType.DeployV1);
    const ci = bonsol_schemas_1.ChannelInstruction.endChannelInstruction(builder);
    builder.finish(ci);
    const buf = builder.asUint8Array();
    return {
        accounts: [
            {
                address: deployer,
                role: web3_js_1.AccountRole.WRITABLE_SIGNER,
            },
            {
                address: payer || deployer,
                role: web3_js_1.AccountRole.WRITABLE_SIGNER,
            },
            {
                address: await (0, _1.deploymentAddress)(imageId),
                role: web3_js_1.AccountRole.WRITABLE,
            },
            {
                address: (0, web3_js_1.address)("11111111111111111111111111111111"),
                role: web3_js_1.AccountRole.READONLY,
            }
        ],
        programAddress: (0, web3_js_1.address)(_1.BONSOL_PROGRAM_ID),
        data: buf,
    };
}
exports.Deploy = Deploy;
//# sourceMappingURL=deploy.js.map