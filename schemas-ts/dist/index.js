"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.ExitCode = void 0;
__exportStar(require("./channel_instruction"), exports);
__exportStar(require("./input_type"), exports);
__exportStar(require("./claim_v1"), exports);
__exportStar(require("./deploy_v1"), exports);
__exportStar(require("./execution_request_v1"), exports);
__exportStar(require("./status_v1"), exports);
var ExitCode;
(function (ExitCode) {
    ExitCode[ExitCode["Success"] = 0] = "Success";
    ExitCode[ExitCode["VerifyError"] = 1] = "VerifyError";
    ExitCode[ExitCode["ProvingError"] = 2] = "ProvingError";
    ExitCode[ExitCode["InputError"] = 3] = "InputError";
})(ExitCode || (exports.ExitCode = ExitCode = {}));
