// Types
export type { WebAuthnCredential, WebAuthnSetupResult } from "./types";

// WebAuthn helpers
export { reuseCredential, setupWebAuthnForSignup } from "./webauthn";

// Popup helpers
export {
  clickAndWaitForPopup,
  getExistingPopup,
  setupPopupLogging,
  waitForPopupToClose,
} from "./popup";

// Service helpers
export type { ServiceConfig } from "./services";
export {
  DEFAULT_SERVICES,
  waitForService,
  waitForServicesToLoad,
} from "./services";
