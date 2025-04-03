import React, { useState, useEffect, useCallback } from 'react';
import {
    View,
    Text,
    StyleSheet,
    TouchableOpacity,
    SafeAreaView,
    ScrollView,
} from 'react-native';
import TransactionFormFields from './TransactionFormFields';
import TransactionFeeView from './TransactionFeeView';
import ToastView from './ToastView';
import ActionButton, { ButtonStyle } from './ActionButton';
import { type PreparedTransaction } from '../../../src';
import { AccountClient } from '../../../src/passkey/authenticate/account_client';
import { loadConfig } from './helpers/loadConfig';

interface SendTransactionViewProps {
    fromAccount: {
        info: {
            domain: string;
            name?: string;
            userID?: string;
        };
        address: string;
        uniqueAccountId: string;
    };
    onTransactionSent?: () => void;
    onDismiss: () => void;
}

const SendTransactionView: React.FC<SendTransactionViewProps> = ({
    fromAccount,
    onTransactionSent = () => { },
    onDismiss
}) => {
    const [toAddress, setToAddress] = useState('0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045');
    const [amount, setAmount] = useState('1.0');
    const [isConfirming, setIsConfirming] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [showingSuccess, setShowingSuccess] = useState(false);
    const [preparedTransaction, setPreparedTransaction] = useState<PreparedTransaction | null>(null);
    const [isPreparing, setIsPreparing] = useState(false);

    const isValidInput = useCallback(() => {
        return toAddress.length > 0 &&
            amount.length > 0 &&
            !isNaN(parseFloat(amount)) &&
            toAddress.startsWith('0x') &&
            toAddress.length === 42;
    }, [toAddress, amount]);

    const prepareTransaction = useCallback(async () => {
        if (!isValidInput()) {
            setPreparedTransaction(null);
            return;
        }

        const amountInEth = parseFloat(amount);
        const amountInWei = Math.floor(amountInEth * 1e18).toString();

        setIsPreparing(true);
        setError(null);

        try {
            const config = loadConfig();
            const accountClient = new AccountClient({
                address: fromAccount.address,
                uniqueAccountId: fromAccount.uniqueAccountId
            }, fromAccount.info.domain, config);

            const prepared = await accountClient.prepareTransaction(toAddress, amountInWei);
            console.log("Prepared transaction:", prepared);
            setPreparedTransaction(prepared);
        } catch (err) {
            console.error("Error preparing transaction:", err);
            setError(err instanceof Error ? err.message : 'Failed to prepare transaction');
            setPreparedTransaction(null);
        } finally {
            setIsPreparing(false);
        }
    }, [toAddress, amount, fromAccount]);

    const confirmTransaction = useCallback(async () => {
        if (!isValidInput()) return;

        const amountInEth = parseFloat(amount);
        const amountInWei = Math.floor(amountInEth * 1e18).toString();

        setIsConfirming(true);
        setError(null);

        try {
            const config = loadConfig();
            const accountClient = new AccountClient({
                address: fromAccount.address,
                uniqueAccountId: fromAccount.uniqueAccountId
            }, fromAccount.info.domain, config);

            await accountClient.sendTransaction(toAddress, amountInWei);
            setShowingSuccess(true);
            onTransactionSent();

            setTimeout(() => {
                onDismiss();
            }, 1500);
        } catch (err) {
            console.error("Error sending transaction:", err);
            setError(err instanceof Error ? err.message : 'Failed to send transaction');
        } finally {
            setIsConfirming(false);
        }
    }, [toAddress, amount, fromAccount, onTransactionSent, onDismiss]);

    useEffect(() => {
        prepareTransaction();
    }, [toAddress, amount]);

    return (
        <SafeAreaView style={styles.container}>
            <View style={styles.header}>
                <Text style={styles.title}>Send Transaction</Text>
                <TouchableOpacity onPress={onDismiss}>
                    <Text style={styles.cancelButton}>Cancel</Text>
                </TouchableOpacity>
            </View>

            <ScrollView style={styles.content}>
                <TransactionFormFields
                    fromAddress={fromAccount.address}
                    toAddress={toAddress}
                    setToAddress={setToAddress}
                    amount={amount}
                    setAmount={setAmount}
                />

                {preparedTransaction && (
                    <TransactionFeeView
                        fee={preparedTransaction.displayFee}
                        isPreparing={isPreparing}
                    />
                )}

                {error && (
                    <View style={styles.errorContainer}>
                        <Text style={styles.errorText}>{error}</Text>
                    </View>
                )}

                <ActionButton
                    title="Send Transaction"
                    progressTitle="Sending Transaction..."
                    icon="paperplane.circle.fill"
                    isLoading={isConfirming || isPreparing}
                    style={ButtonStyle.Prominent}
                    action={confirmTransaction}
                />
            </ScrollView>
            
            {showingSuccess && (
                <View style={styles.toastOverlay}>
                    <ToastView
                        message="Transaction Sent!"
                        icon="✓"
                        iconColor="#4CD964"
                    />
                </View>
            )}
        </SafeAreaView>
    );
};

const styles = StyleSheet.create({
    container: {
        flex: 1,
        backgroundColor: '#FFFFFF',
    },
    header: {
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: 16,
        borderBottomWidth: 1,
        borderBottomColor: '#E0E0E0',
    },
    title: {
        fontSize: 18,
        fontWeight: 'bold',
    },
    cancelButton: {
        fontSize: 16,
        color: '#007AFF',
    },
    content: {
        flex: 1,
        padding: 16,
    },
    errorContainer: {
        padding: 16,
        backgroundColor: 'rgba(255, 0, 0, 0.05)',
        borderRadius: 8,
        marginBottom: 20,
    },
    errorText: {
        color: 'red',
    },
    toastOverlay: {
        position: 'absolute',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.5)',
        justifyContent: 'center',
        alignItems: 'center',
    },
});

export default SendTransactionView; 