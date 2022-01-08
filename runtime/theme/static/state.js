const TOKEN_KIND_STRING = "STRING";
const TOKEN_KIND_EQUALS = "EQUALS";
const TOKEN_KIND_IDENT = "IDENT";

const MAGIC_PREFIX = "#csai:";

export default class StateObserver {
    constructor() {
        this.listeners = [];
        this.errorCallback = null;

        window.addEventListener("hashchange", () => { this.trigger() });
    }

    onAction(action, args, callback) {
        this.listeners.push({
            action: action,
            args: args,
            callback: callback,
        })
    }

    onError(callback) {
        this.errorListener = callback;
    }

    _emitError(error) {
        if (this.errorListener) {
            this.errorListener(error);
        }
    }

    trigger() {
        let stateStr = decodeURIComponent(window.location.hash);
        if (stateStr.startsWith(MAGIC_PREFIX)) {
            stateStr = stateStr.substring(MAGIC_PREFIX.length);

            let state;
            try {
                state = parseState(stateStr);
            } catch (e) {
                this._emitError(e.message);
                return;
            }

            // find listener that corresponds to this action
            let action = state.action;
            let listener;
            for (let i = 0; i < this.listeners.length; i++) {
                if (this.listeners[i].action == action) {
                    listener = this.listeners[i];
                    break;
                }
            }

            if (!listener) {
                return;
            }

            // get args asked by the listener
            let args = []
            for (let i = 0; i < listener.args.length; i++) {
                let arg = state.getArg(listener.args[i]);
                if (arg) {
                    args.push(arg);
                } else {
                    this._emitError(`expected parameter \`${listener.args[i]}\``);
                    return;
                }
            }

            listener.callback(args);
        }
    }
}

class Token {
    constructor(kind, value) {
        this.kind = kind;
        this.value = value;
    }
}

class Argument {
    constructor(parameter, value) {
        this.parameter = parameter;
        this.value = value;
    }
}

class State {
    constructor(action, args) {
        this.action = action;
        this.args = args;
    }

    getArg(parameter) {
        for (const arg of this.args) {
            if (arg.parameter == parameter) {
                return arg.value;
            }
        }
        return null;
    }
}

function parseState(state) {
    let chars = [];

    for (const char of state) {
        chars.push(char);
    }

    let tokens = [];

    while (chars.length != 0) {
        chars = eatTrivia(chars);
        if (chars[0] == "\"") { // string start
            chars = tokenizeString(chars, tokens);
        } else if (chars[0] == "=") { // equals
            tokens.push(new Token(TOKEN_KIND_EQUALS, "="));
            chars = chars.slice(1);
        } else { // identifier
            chars = tokenizeIdent(chars, tokens);
        }
    }

    if (tokens.length == 0) {
        throw new Error(`state can't be empty`);
    }

    if (tokens[0].kind != TOKEN_KIND_IDENT) {
        throw new Error(`expected state name but got \`${tokens[0].value}\``);
    }

    let action = tokens[0];
    let args = [];

    tokens = tokens.slice(1);
    while (tokens.length != 0) {
        if (tokens[0].kind != TOKEN_KIND_IDENT) {
            throw new Error(`expected parameter name but got \`${tokens[0].value}\``);
        }
        let param = tokens[0];

        if (tokens[1].kind != TOKEN_KIND_EQUALS) {
            throw new Error(`expected equals but got \`${tokens[1].value}\``);
        }

        if (tokens[2].kind != TOKEN_KIND_STRING) {
            throw new Error(`expected string argument but got \`${tokens[2].value}\``);
        }
        let argValue = tokens[2];

        args.push(new Argument(param.value, argValue.value));

        tokens = tokens.slice(3);
    }

    return new State(action.value, args);
}

function eatTrivia(chars) {
    let i = 0;

    for (i = 0; i < chars.length; i++) {
        if (chars[i] != " " && chars[i] != "/" && chars[i] != "," && chars[i] != "?") {
            break;
        }
    }

    return chars.slice(i);
}

function tokenizeString(chars, tokens) {
    let escaping = false;
    let value = "";
    let i = 0;

    for (i = 1; i < chars.length; i++) {
        if (escaping) { // escape sequences
            switch (chars[i]) {
                case "\"":
                case "~":
                    value += chars[i];
                    escaping = false;
                    break;
                default:
                    throw new Error(`invalid escape sequence: \\${chars[i]}`);
            }
        } else if (chars[i] == "\"") { // string end
            break;
        } else if (chars[i] == "~") { // escape sequence start
            escaping = true;
        } else { // other
            value += chars[i];
            escaping = false;
        }
    }

    tokens.push(new Token(TOKEN_KIND_STRING, value))

    return chars.slice(i+1);
}

function tokenizeIdent(chars, tokens) {
    let value = "";
    let i = 0;

    for (i = 0; i < chars.length; i++) {
        if (chars[i] == " " || chars[i] == "=" || chars[i] == "/" || chars[i] == "," || chars[i] == "?") {
            break;
        }
        value += chars[i];
    }

    tokens.push(new Token(TOKEN_KIND_IDENT, value));

    return chars.slice(i);
}
