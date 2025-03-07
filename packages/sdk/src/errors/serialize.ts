import { LIB_VERSION } from "../version.js";
import { standardErrorCodes } from "./constants.js";
import { serialize, type SerializedEthereumRpcError } from "./utils.js";

type ErrorResponse = {
  method: unknown;
  errorCode?: number;
  errorMessage: string;
};

/**
 * Serializes an error to a format that is compatible with the Ethereum JSON RPC error format.
 */
export function serializeError(
  error: unknown,
  requestOrMethod?: JSONRPCRequest | JSONRPCRequest[] | string,
): SerializedError {
  const serialized = serialize(getErrorObject(error), {
    shouldIncludeStack: true,
  });

  const docUrl = new URL("https://docs.zksync.io/zksync-account-sdk/docs/errors");
  docUrl.searchParams.set("version", LIB_VERSION);
  docUrl.searchParams.set("code", serialized.code.toString());
  const method = getMethod(serialized.data, requestOrMethod);
  if (method) {
    docUrl.searchParams.set("method", method);
  }
  docUrl.searchParams.set("message", serialized.message);

  return {
    ...serialized,
    docUrl: docUrl.href,
  };
}

function isErrorResponse(response: unknown): response is ErrorResponse {
  return (response as ErrorResponse).errorMessage !== undefined;
}

/**
 * Converts an error to a serializable object.
 */
function getErrorObject(error: unknown) {
  if (typeof error === "string") {
    return {
      message: error,
      code: standardErrorCodes.rpc.internal,
    };
  } else if (isErrorResponse(error)) {
    return {
      ...error,
      message: error.errorMessage,
      code: error.errorCode,
      data: { method: error.method },
    };
  }
  return error;
}

/**
 * Gets the method name from the serialized data or the request.
 */
function getMethod(
  serializedData: unknown,
  request?: JSONRPCRequest | JSONRPCRequest[] | string,
): string | undefined {
  const methodInData = (serializedData as { method: string })?.method;
  if (methodInData) {
    return methodInData;
  }

  if (request === undefined) {
    return undefined;
  } else if (typeof request === "string") {
    return request;
  } else if (!Array.isArray(request)) {
    return request.method;
  } else if (request.length > 0) {
    return request[0]!.method;
  }
  return undefined;
}

interface SerializedError extends SerializedEthereumRpcError {
  docUrl: string;
}

interface JSONRPCRequest {
  method: string;
}
