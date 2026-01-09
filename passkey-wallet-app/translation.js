import i18next from "i18next";

export const ENGLISH = {
  translation: {
    topbar: {
      network: "Network",
      balance: "Balance",
    },
    nav: {
      home: "Home",
      send: "Send",
      earn: "Earn",
      interop: "Interop",
      activity: "Activity",
    },
    home: {
      getStarted: "Get Started",
      setupWallet: "Set up your secure PayPal wallet to start sending and receiving money",
      createKey: "Create Security Key",
      userName: "Your Name",
      createPasskeyBtn: "Create Secure Wallet",
      keyCreated: "✓ Security Key Created",
      credentialID: "Credential ID:",
      resetPasskeyBtn: "Reset",
      activateWallet: "Activate Your Wallet",
      deployWallet: "Deploy your smart wallet on the blockchain to start managing your funds.",
      deployAccountBtn: "Activate Wallet",
      walletActivated: "✓ Wallet Activated!",
      walletAddress: "Address:",
      refreshBalanceBtn: "Refresh Balance",
      walletBalance: "Your Balance",
      yourWallet: "Your Wallet",
      resetPasskeyMainBtn: "Reset Passkey",
      quickActions: "Quick Actions",
      quickSendBtn: "Send Money",
      quickEarnBtn: "Earn Interest",
      faucetBtn: "Faucet for your wallet",
    },
    send: {
      sendMoney: "Send Money",
      recipientAddress: "Recipient Address",
      amount: "Amount (ETH)",
      transferBtn: "Send ETH",
      txSent: "✓ Transaction Sent!",
      txLabel: "Transaction:",
    },
    earn: {
      title: "Earn Interest with Aave",
      subtitle: "Deposit your ETH to earn interest through Aave lending protocol on Ethereum Layer 1.",
      shadowAccount: "Shadow Account:",
      deposits: "Aave Deposits",
      refreshBtn: "Refresh",
      depositLabel: "Deposit",
      depositAmount: "Amount (ETH)",
      depositBtn: "Deposit to Aave",
      depositInit: "✓ Deposit Initiated!",
      depositTx: "Transaction:",
      depositWaitMsg: "⏳ Your deposit will be finalized in ~15 minutes",
      withdrawLabel: "Withdraw",
      withdrawAmount: "Amount (aETH)",
      withdrawBtn: "Withdraw from Aave",
      withdrawInit: "✓ Withdrawal Initiated!",
      withdrawTx: "Transaction:",
      withdrawWaitMsg: "⏳ Your withdrawal will be finalized in ~15 minutes",
    },
    interop: {
      title: "Cross-Chain Interop (Local)",
      subtitle: "Send messages between two local ZKsync chains using L2-to-L2 interop messaging.",
      noteLabel: "Note:",
      note: "This feature connects to local chains only (ports 3050 & 3051).",
      setupWarning: "Setup Required",
      setupMsg: "Make sure you have two local ZKsync chains running:",
      chainA: "Chain A:",
      chainB: "Chain B:",
      connectionIssue: "Connection Issue",
      connectionMsg: "Please check that:",
      conectionCheck1: "Both local chains are running (ports 3050 & 3051)",
      conectionCheck2: "The token is deployed on Chain A",
      sendMsgLabel: "Send Interop Message",
      msgLabel: "Message",
      msgDirection: "Message Direction",
      aToB: "Chain A → Chain B",
      bToA: "Chain B → Chain A",
      sendMsgBtn: "Send Message",
      msgProgressTitle: "⏳ Sending message...",
      msgVerified: "✓ Interop Message Verified!",
      srcChainTx: "Source Chain TX:",
      l1Batch: "L1 Batch:",
      msgIndex: "Message Index:",
      msgFinalDirection: "Direction:",
      msgVerifiedOnDestination: "Verified on Destination:",
      msgSuccess: "✓ Success",
      tokenTransferTitle: "Cross-Chain Token Transfer",
      tokenTransferSubtitle: "Transfer ERC20 tokens between Chain A and Chain B",
      tokenLabel: "Token:",
      tokenAddress: "Address:",
      chainABalance: "Balance on Chain A:",
      chainBBalance: "Balance on Chain B:",
      refreshTokenBalanceBtn: "Refresh Token Balances",
      tokenTransferAmount: "Amount to Transfer",
      transferDirection: "Transfer Direction",
      transferTokenAToB: "Chain A → Chain B",
      transferTokenBToA: "Chain B → Chain A",
      transferBtn: "Transfer Tokens",
      transferProgressTitle: "⏳ Transferring tokens...",
      transferComplete: "✓ Token Transfer Complete!",
      statusSrcChainTx: "Source Chain TX:",
      destinationChainTx: "Destination Chain TX:",
      statusAmount: "Amount:",
      statusDirection: "Direction:",
    },
    activity: {
      title: "Recent Activity",
      subtitle: "Your transaction history will appear here",
      noTxns: "No recent transactions",
    },
    footer: {
      network: "Network:",
      poweredByZk: "Powered by ZKSync",
      securedByPasskeys: "Secured by Passkey Authentication",
    },
  },
};

export const SPANISH = {
  translation: {
    topbar: {
      network: "Red",
      balance: "Saldo",
    },
    nav: {
      home: "Inicio",
      send: "Enviar",
      earn: "Generar",
      interop: "Interop",
      activity: "Actividad",
    },
    home: {
      getStarted: "Empezar",
      setupWallet:
        "Configura tu billetera segura de PayPal para empezar a enviar y recibir dinero",
      createKey: "Crear clave de seguridad",
      userName: "Tu nombre",
      createPasskeyBtn: "Crear billetera segura",
      keyCreated: "✓ Clave de seguridad creada",
      credentialID: "ID de credencial:",
      resetPasskeyBtn: "Reiniciar",
      activateWallet: "Activa tu billetera",
      deployWallet:
        "Implementa tu monedero inteligente en la red para empezar a gestionar tus fondos.",
      deployAccountBtn: "Activar billetera",
      walletActivated: "✓ ¡Billetera activada!",
      walletAddress: "Dirección:",
      refreshBalanceBtn: "Actualizar saldo",
      walletBalance: "Tu saldo",
      yourWallet: "Tu billetera",
      resetPasskeyMainBtn: "Restablecer passkey",
      quickActions: "Acciones rápidas",
      quickSendBtn: "Enviar dinero",
      quickEarnBtn: "Genera intereses",
      faucetBtn: "Faucet para tu billetera",
    },
    send: {
      sendMoney: "Enviar dinero",
      recipientAddress: "Dirección del destinatario",
      amount: "Cantidad (ETH)",
      transferBtn: "Enviar ETH",
      txSent: "✓ ¡Transacción enviada!",
      txLabel: "Transacción:",
    },
    earn: {
      title: "Genera intereses con Aave",
      subtitle:
        "Deposita tu ETH para generar intereses a través del protocolo de préstamos Aave en Ethereum Layer 1.",
      shadowAccount: "Cuenta sombra:",
      deposits: "Depósitos en Aave",
      refreshBtn: "Actualizar",
      depositLabel: "Depositar",
      depositAmount: "Cantidad (ETH)",
      depositBtn: "Depositar en Aave",
      depositInit: "✓ ¡Depósito iniciado!",
      depositTx: "Transacción:",
      depositWaitMsg: "⏳ Tu depósito se confirmará en ~15 minutos",
      withdrawLabel: "Retirar",
      withdrawAmount: "Cantidad (aETH)",
      withdrawBtn: "Retirar de Aave",
      withdrawInit: "✓ ¡Retiro iniciado!",
      withdrawTx: "Transacción:",
      withdrawWaitMsg: "⏳ Tu retiro se confirmará en ~15 minutos",
    },
    interop: {
      title: "Interop entre redes (Local)",
      subtitle:
        "Envía mensajes entre dos redes locales de ZKsync usando mensajería interop L2-a-L2.",
      noteLabel: "Nota:",
      note: "Esta función solo se conecta a redes locales (puertos 3050 y 3051).",
      setupWarning: "Configuración requerida",
      setupMsg: "Asegúrate de tener dos redes locales de ZKsync en ejecución:",
      chainA: "Red A:",
      chainB: "Red B:",
      connectionIssue: "Problema de conexión",
      connectionMsg: "Por favor, compruebe que:",
      conectionCheck1: "Ambas redes locales están funcionando (puertos 3050 y 3051)",
      conectionCheck2: "El token se implementa en la Red A",
      sendMsgLabel: "Enviar mensaje interop",
      msgLabel: "Mensaje",
      msgDirection: "Dirección del mensaje",
      aToB: "Red A → Red B",
      bToA: "Red B → Red A",
      sendMsgBtn: "Enviar mensaje",
      msgProgressTitle: "⏳ Enviando mensaje...",
      msgVerified: "✓ ¡Mensaje de interop verificado!",
      srcChainTx: "TX de red de origen:",
      l1Batch: "Lote L1:",
      msgIndex: "Índice de mensajes:",
      msgFinalDirection: "Dirección:",
      msgVerifiedOnDestination: "Verificado en el destino:",
      msgSuccess: "✓ Éxito",
      tokenTransferTitle: "Transferencia de tokens entre redes",
      tokenTransferSubtitle: "Transfiere tokens ERC20 entre la Red A y la Red B",
      tokenLabel: "Token:",
      tokenAddress: "Dirección:",
      chainABalance: "Saldo en la Red A:",
      chainBBalance: "Saldo en la Red B:",
      refreshTokenBalanceBtn: "Actualizar saldos de tokens",
      tokenTransferAmount: "Cantidad a transferir",
      transferDirection: "Dirección de la transferencia",
      transferTokenAToB: "Red A → Red B",
      transferTokenBToA: "Red B → Red A",
      transferBtn: "Transferir tokens",
      transferProgressTitle: "⏳ Transfiriendo tokens...",
      transferComplete: "✓ ¡Transferencia de token completada!",
      statusSrcChainTx: "TX de red de origen:",
      destinationChainTx: "TX de red de destino:",
      statusAmount: "Cantidad:",
      statusDirection: "Dirección:",
    },
    activity: {
      title: "Actividad reciente",
      subtitle: "Tu historia de transacciones aparecerá aquí",
      noTxns: "No hay transacciones recientes",
    },
    footer: {
      network: "Red:",
      poweredByZk: "Impulsado por ZKsync",
      securedByPasskeys: "Protegido con autenticación por passkey",
    },
  },
};

export const PORTUGUESE_BR = {
  translation: {
    topbar: {
      network: "Rede",
      balance: "Saldo",
    },
    nav: {
      home: "Início",
      send: "Enviar",
      earn: "Ganhar",
      interop: "Interop",
      activity: "Atividade",
    },
    home: {
      getStarted: "Começar",
      setupWallet:
        "Configure sua carteira segura PayPal para começar a enviar e receber dinheiro",
      createKey: "Criar chave de segurança",
      userName: "Seu nome",
      createPasskeyBtn: "Criar carteira segura",
      keyCreated: "✓ Chave de segurança criada",
      credentialID: "ID da credencial:",
      resetPasskeyBtn: "Redefinir",
      activateWallet: "Ative sua carteira",
      deployWallet:
        "Implante sua carteira inteligente na blockchain para começar a gerenciar seus fundos.",
      deployAccountBtn: "Ativar carteira",
      walletActivated: "✓ Carteira ativada!",
      walletAddress: "Endereço:",
      refreshBalanceBtn: "Atualizar saldo",
      walletBalance: "Seu saldo",
      yourWallet: "Sua carteira",
      resetPasskeyMainBtn: "Redefinir passkey",
      quickActions: "Ações rápidas",
      quickSendBtn: "Enviar dinheiro",
      quickEarnBtn: "Ganhar juros",
      faucetBtn: "Faucet para sua carteira",
    },
    send: {
      sendMoney: "Enviar dinheiro",
      recipientAddress: "Endereço do destinatário",
      amount: "Quantidade (ETH)",
      transferBtn: "Enviar ETH",
      txSent: "✓ Transação enviada!",
      txLabel: "Transação:",
    },
    earn: {
      title: "Ganhe juros com a Aave",
      subtitle:
        "Deposite seu ETH para gerar juros através do protocolo de empréstimos da Aave na Ethereum Layer 1.",
      shadowAccount: "Conta sombra:",
      deposits: "Depósitos na Aave",
      refreshBtn: "Atualizar",
      depositLabel: "Depositar",
      depositAmount: "Quantidade (ETH)",
      depositBtn: "Depositar na Aave",
      depositInit: "✓ Depósito iniciado!",
      depositTx: "Transação:",
      depositWaitMsg: "⏳ Seu depósito será confirmado em ~15 minutos",
      withdrawLabel: "Sacar",
      withdrawAmount: "Quantidade (aETH)",
      withdrawBtn: "Sacar da Aave",
      withdrawInit: "✓ Saque iniciado!",
      withdrawTx: "Transação:",
      withdrawWaitMsg: "⏳ Seu saque será confirmado em ~15 minutos",
    },
    interop: {
      title: "Interop entre redes (local)",
      subtitle:
        "Envie mensagens entre duas redes locais da ZKsync usando mensageria interop L2-para-L2.",
      noteLabel: "Nota:",
      note: "Este recurso conecta apenas a redes locais (portas 3050 e 3051).",
      setupWarning: "Configuração necessária",
      setupMsg: "Certifique-se de ter duas redes locais da ZKsync em execução:",
      chainA: "Rede A:",
      chainB: "Rede B:",
      connectionIssue: "Problema de conexão",
      connectionMsg: "Por favor, verifique se:",
      conectionCheck1: "Ambas as redes locais estão funcionando (portas 3050 e 3051)",
      conectionCheck2: "O token está implantado na Rede A",
      sendMsgLabel: "Enviar mensagem interop",
      msgLabel: "Mensagem",
      msgDirection: "Direção da mensagem",
      aToB: "Rede A → Rede B",
      bToA: "Rede B → Rede A",
      sendMsgBtn: "Enviar mensagem",
      msgProgressTitle: "⏳ Enviando mensagem...",
      msgVerified: "✓ Mensagem interop verificada!",
      srcChainTx: "TX da rede de origem:",
      l1Batch: "Lote L1:",
      msgIndex: "Índice de mensagens:",
      msgFinalDirection: "Direção:",
      msgVerifiedOnDestination: "Verificado no destino:",
      msgSuccess: "✓ Sucesso",
      tokenTransferTitle: "Transferência de tokens entre redes",
      tokenTransferSubtitle: "Transfira tokens ERC20 entre a Rede A e a Rede B",
      tokenLabel: "Token:",
      tokenAddress: "Endereço:",
      chainABalance: "Saldo na Rede A:",
      chainBBalance: "Saldo na Rede B:",
      refreshTokenBalanceBtn: "Atualizar saldos de tokens",
      tokenTransferAmount: "Quantidade a transferir",
      transferDirection: "Direção da transferência",
      transferTokenAToB: "Rede A → Rede B",
      transferTokenBToA: "Rede B → Rede A",
      transferBtn: "Transferir tokens",
      transferProgressTitle: "⏳ Transferindo tokens...",
      transferComplete: "✓ Transferência de token concluída!",
      statusSrcChainTx: "TX da rede de origem:",
      destinationChainTx: "TX da rede de destino:",
      statusAmount: "Quantidade:",
      statusDirection: "Direção:",
    },
    activity: {
      title: "Atividade recente",
      subtitle: "Seu histórico de transações aparecerá aqui",
      noTxns: "Nenhuma transação recente",
    },
    footer: {
      network: "Rede:",
      poweredByZk: "Desenvolvido com ZKSync",
      securedByPasskeys: "Protegido por autenticação via passkey",
    },
  },
};

export default async function setupTranslation(language) {
  await i18next.init({
    lng: language,
    debug: true,
    fallback: "en",
    resources: {
      en: ENGLISH,
      es: SPANISH,
      pt: PORTUGUESE_BR,
    },
  });

  // TOP BAR
  document.getElementById("topbar-network").innerHTML = i18next.t("topbar.network");
  document.getElementById("topbar-balance").innerHTML = i18next.t("topbar.balance");

  // NAVIGATION
  document.getElementById("nav-home").innerHTML = i18next.t("nav.home");
  document.getElementById("nav-send").innerHTML = i18next.t("nav.send");
  document.getElementById("nav-earn").innerHTML = i18next.t("nav.earn");
  document.getElementById("nav-interop").innerHTML = i18next.t("nav.interop");
  document.getElementById("nav-activity").innerHTML = i18next.t("nav.activity");

  // HOME TAB
  document.getElementById("home-get-started").innerHTML = i18next.t("home.getStarted");
  document.getElementById("home-setup-wallet").innerHTML = i18next.t("home.setupWallet");
  document.getElementById("home-create-key").innerHTML = i18next.t("home.createKey");
  document.getElementById("home-user-name").innerHTML = i18next.t("home.userName");
  document.getElementById("createPasskeyBtn").innerHTML = i18next.t("home.createPasskeyBtn");
  document.getElementById("home-key-created").innerHTML = i18next.t("home.keyCreated");
  document.getElementById("home-credential-id").innerHTML = i18next.t("home.credentialID");
  document.getElementById("resetPasskeyBtn").innerHTML = i18next.t("home.resetPasskeyBtn");
  document.getElementById("home-activate-wallet").innerHTML = i18next.t("home.activateWallet");
  document.getElementById("home-deploy-wallet").innerHTML = i18next.t("home.deployWallet");
  document.getElementById("deployAccountBtn").innerHTML = i18next.t("home.deployAccountBtn");
  document.getElementById("home-wallet-activated").innerHTML = i18next.t("home.walletActivated");
  document.getElementById("home-wallet-address").innerHTML = i18next.t("home.walletAddress");
  document.getElementById("refreshBalanceBtn").innerHTML = i18next.t("home.refreshBalanceBtn");
  document.getElementById("home-wallet-balance").innerHTML = i18next.t("home.walletBalance");
  document.getElementById("home-your-wallet").innerHTML = i18next.t("home.yourWallet");
  document.getElementById("home-wallet-address-2").innerHTML = i18next.t("home.walletAddress");
  document.getElementById("resetPasskeyMainBtn").innerHTML = i18next.t("home.resetPasskeyMainBtn");
  document.getElementById("home-quick-actions").innerHTML = i18next.t("home.quickActions");
  document.getElementById("quickSendBtn").innerHTML = i18next.t("home.quickSendBtn");
  document.getElementById("quickEarnBtn").innerHTML = i18next.t("home.quickEarnBtn");
  document.getElementById("faucetBtn").innerHTML = i18next.t("home.faucetBtn");

  // SEND TAB
  document.getElementById("send-money").innerHTML = i18next.t("send.sendMoney");
  document.getElementById("send-recipient").innerHTML = i18next.t("send.recipientAddress");
  document.getElementById("send-amount").innerHTML = i18next.t("send.amount");
  document.getElementById("transferBtn").innerHTML = i18next.t("send.transferBtn");
  document.getElementById("send-tx-sent").innerHTML = i18next.t("send.txSent");
  document.getElementById("send-tx-label").innerHTML = i18next.t("send.txLabel");

  // EARN TAB
  document.getElementById("earn-title").innerHTML = i18next.t("earn.title");
  document.getElementById("earn-subtitle").innerHTML = i18next.t("earn.subtitle");
  document.getElementById("earn-shadow").innerHTML = i18next.t("earn.shadowAccount");
  document.getElementById("earn-deposits").innerHTML = i18next.t("earn.deposits");
  document.getElementById("refreshAaveBalanceBtn").innerHTML = i18next.t("earn.refreshBtn");
  document.getElementById("earn-deposit-label").innerHTML = i18next.t("earn.depositLabel");
  document.getElementById("earn-deposit-amount").innerHTML = i18next.t("earn.depositAmount");
  document.getElementById("aaveDepositBtn").innerHTML = i18next.t("earn.depositBtn");
  document.getElementById("earn-deposit-init").innerHTML = i18next.t("earn.depositInit");
  document.getElementById("earn-deposit-tx").innerHTML = i18next.t("earn.depositTx");
  document.getElementById("earn-deposit-wait-msg").innerHTML = i18next.t("earn.depositWaitMsg");
  document.getElementById("earn-withdraw-label").innerHTML = i18next.t("earn.withdrawLabel");
  document.getElementById("earn-withdraw-amount").innerHTML = i18next.t("earn.withdrawAmount");
  document.getElementById("aaveWithdrawBtn").innerHTML = i18next.t("earn.withdrawBtn");
  document.getElementById("earn-withdraw-init").innerHTML = i18next.t("earn.withdrawInit");
  document.getElementById("earn-withdraw-tx").innerHTML = i18next.t("earn.withdrawTx");
  document.getElementById("earn-withdraw-wait-msg").innerHTML = i18next.t("earn.withdrawWaitMsg");

  // INTEROP TAB
  document.getElementById("interop-title").innerHTML = i18next.t("interop.title");
  document.getElementById("interop-subtitle").innerHTML = i18next.t("interop.subtitle");
  document.getElementById("interop-note-label").innerHTML = i18next.t("interop.noteLabel");
  document.getElementById("interop-note").innerHTML = i18next.t("interop.note");
  document.getElementById("interop-setup-warning").innerHTML = i18next.t("interop.setupWarning");
  document.getElementById("interop-setup-msg").innerHTML = i18next.t("interop.setupMsg");
  document.getElementById("interop-chain-a").innerHTML = i18next.t("interop.chainA");
  document.getElementById("interop-chain-b").innerHTML = i18next.t("interop.chainB");
  document.getElementById("interop-connection-issue").innerHTML = i18next.t("interop.connectionIssue");
  document.getElementById("interop-connection-msg").innerHTML = i18next.t("interop.connectionMsg");
  document.getElementById("interop-connection-check-1").innerHTML = i18next.t("interop.conectionCheck1");
  document.getElementById("interop-connection-check-2").innerHTML = i18next.t("interop.conectionCheck2");
  document.getElementById("interop-send-msg-label").innerHTML = i18next.t("interop.sendMsgLabel");
  document.getElementById("interop-msg-label").innerHTML = i18next.t("interop.msgLabel");
  document.getElementById("interop-msg-direction").innerHTML = i18next.t("interop.msgDirection");
  document.getElementById("interop-a-to-b").innerHTML = i18next.t("interop.aToB");
  document.getElementById("interop-b-to-a").innerHTML = i18next.t("interop.bToA");
  document.getElementById("interopSendBtn").innerHTML = i18next.t("interop.sendMsgBtn");
  document.getElementById("interopProgressTitle").innerHTML = i18next.t("interop.msgProgressTitle");
  document.getElementById("interop-msg-verified").innerHTML = i18next.t("interop.msgVerified");
  document.getElementById("interop-src-chain-tx").innerHTML = i18next.t("interop.srcChainTx");
  document.getElementById("interop-l1-batch").innerHTML = i18next.t("interop.l1Batch");
  document.getElementById("interop-msg-index").innerHTML = i18next.t("interop.msgIndex");
  document.getElementById("interop-msg-final-direction").innerHTML = i18next.t("interop.msgFinalDirection");
  document.getElementById("interop-msg-verified-on-dest").innerHTML = i18next.t("interop.msgVerifiedOnDestination");
  document.getElementById("interop-msg-success").innerHTML = i18next.t("interop.msgSuccess");
  document.getElementById("interop-token-transfer-title").innerHTML = i18next.t("interop.tokenTransferTitle");
  document.getElementById("interop-token-transfer-subtitle").innerHTML = i18next.t("interop.tokenTransferSubtitle");
  document.getElementById("interop-token-label").innerHTML = i18next.t("interop.tokenLabel");
  document.getElementById("interop-token-address").innerHTML = i18next.t("interop.tokenAddress");
  document.getElementById("interop-chain-a-balance").innerHTML = i18next.t("interop.chainABalance");
  document.getElementById("interop-chain-b-balance").innerHTML = i18next.t("interop.chainBBalance");
  document.getElementById("refreshTokenBalancesBtn").innerHTML = i18next.t("interop.refreshTokenBalanceBtn");
  document.getElementById("interop-token-transfer-amount").innerHTML = i18next.t("interop.tokenTransferAmount");
  document.getElementById("interop-transfer-direction").innerHTML = i18next.t("interop.transferDirection");
  document.getElementById("interop-transfer-token-a-to-b").innerHTML = i18next.t("interop.transferTokenAToB");
  document.getElementById("interop-transfer-token-b-to-a").innerHTML = i18next.t("interop.transferTokenBToA");
  document.getElementById("tokenTransferBtn").innerHTML = i18next.t("interop.transferBtn");
  document.getElementById("tokenTransferProgressTitle").innerHTML = i18next.t("interop.transferProgressTitle");
  document.getElementById("interop-transfer-complete").innerHTML = i18next.t("interop.transferComplete");
  document.getElementById("interop-status-src-chain-tx").innerHTML = i18next.t("interop.statusSrcChainTx");
  document.getElementById("interop-destination-chain-tx").innerHTML = i18next.t("interop.destinationChainTx");
  document.getElementById("interop-status-amount").innerHTML = i18next.t("interop.statusAmount");
  document.getElementById("interop-status-direction").innerHTML = i18next.t("interop.statusDirection");

  // ACTIVITY TAB
  document.getElementById("activity-title").innerHTML = i18next.t("activity.title");
  document.getElementById("activity-subtitle").innerHTML = i18next.t("activity.subtitle");
  document.getElementById("activity-no-txns").innerHTML = i18next.t("activity.noTxns");

  // FOOTER
  document.getElementById("footer-network").innerHTML = i18next.t("footer.network");
  document.getElementById("footer-powered-by-zk").innerHTML = i18next.t("footer.poweredByZk");
  document.getElementById("footer-secured-by-passkeys").innerHTML = i18next.t("footer.securedByPasskeys");
}
