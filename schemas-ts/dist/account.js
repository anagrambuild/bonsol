"use strict";
// automatically generated by the FlatBuffers compiler, do not modify
Object.defineProperty(exports, "__esModule", { value: true });
exports.AccountT = exports.Account = void 0;
class Account {
    constructor() {
        this.bb = null;
        this.bb_pos = 0;
    }
    __init(i, bb) {
        this.bb_pos = i;
        this.bb = bb;
        return this;
    }
    writable() {
        return this.bb.readUint8(this.bb_pos);
    }
    mutate_writable(value) {
        this.bb.writeUint8(this.bb_pos + 0, value);
        return true;
    }
    pubkey(index) {
        return this.bb.readUint8(this.bb_pos + 1 + index);
    }
    static sizeOf() {
        return 40;
    }
    static createAccount(builder, writable, pubkey) {
        var _a;
        builder.prep(8, 40);
        builder.pad(7);
        for (let i = 31; i >= 0; --i) {
            builder.writeInt8(((_a = pubkey === null || pubkey === void 0 ? void 0 : pubkey[i]) !== null && _a !== void 0 ? _a : 0));
        }
        builder.writeInt8(writable);
        return builder.offset();
    }
    unpack() {
        return new AccountT(this.writable(), this.bb.createScalarList(this.pubkey.bind(this), 32));
    }
    unpackTo(_o) {
        _o.writable = this.writable();
        _o.pubkey = this.bb.createScalarList(this.pubkey.bind(this), 32);
    }
}
exports.Account = Account;
class AccountT {
    constructor(writable = 0, pubkey = []) {
        this.writable = writable;
        this.pubkey = pubkey;
    }
    pack(builder) {
        return Account.createAccount(builder, this.writable, this.pubkey);
    }
}
exports.AccountT = AccountT;
