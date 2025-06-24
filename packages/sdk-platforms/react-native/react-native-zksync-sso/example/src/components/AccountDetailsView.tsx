import React, { useState, useEffect } from 'react';
import { View, Text, StyleSheet, ScrollView, TouchableOpacity, Alert, Pressable, SafeAreaView, StatusBar, Linking, Modal } from 'react-native';
import type { AccountDetails } from './types.ts';
import ActionButton, { ButtonStyle } from './ActionButton';
import { loadBalance } from './helpers/loadBalance';
import { fundAccount } from './helpers/fundAccount';
import SendTransactionView from './SendTransactionView';
import AddressView from './AddressView';

interface AccountDetailsViewProps {
    account: AccountDetails;
    onLogout?: () => void;
}

const AccountDetailsView: React.FC<AccountDetailsViewProps> = ({
    account,
    onLogout
}) => {
    const [isLoadingBalance, setIsLoadingBalance] = useState(true);
    const [isFunding, setIsFunding] = useState(false);
    const [showingCopiedFeedback, setShowingCopiedFeedback] = useState(false);
    const [showingSendTransaction, setShowingSendTransaction] = useState(false);
    const [showingLogoutConfirmation, setShowingLogoutConfirmation] = useState(false);
    const [balance, setBalance] = useState<string | null>(account.balance || null);

    // Load balance using the helper function
    const fetchBalance = async () => {
        if (balance) {
            setIsLoadingBalance(false);
            return;
        }

        setIsLoadingBalance(true);
        const fetchedBalance = await loadBalance(account.address);
        setBalance(fetchedBalance);
        setIsLoadingBalance(false);
    };

    // Fund account using the helper function
    const handleFundAccount = async () => {
        if (isFunding) return;

        setIsFunding(true);
        try {
            await fundAccount(account.address);

            const newBalance = await loadBalance(account.address);
            setBalance(newBalance);
        } catch (error) {
            console.error('Error funding account:', error);
        } finally {
            setIsFunding(false);
        }
    };

    // Handle transaction sent
    const handleTransactionSent = async () => {
        // Reload the balance after a transaction is sent
        const newBalance = await loadBalance(account.address);
        setBalance(newBalance);
    };

    // Simulate copy to clipboard and show feedback
    const copyToClipboard = (text: string) => {
        // In a real app, would use Clipboard API
        console.log('Copied to clipboard:', text);
        setShowingCopiedFeedback(true);
        setTimeout(() => {
            setShowingCopiedFeedback(false);
        }, 2000);
    };

    // Show logout confirmation
    const confirmLogout = () => {
        Alert.alert(
            'Are you sure you want to log out?',
            'You can sign back in using your passkey.',
            [
                {
                    text: 'Cancel',
                    style: 'cancel',
                },
                {
                    text: 'Logout',
                    style: 'destructive',
                    onPress: () => onLogout && onLogout(),
                },
            ]
        );
    };

    // Open explorer URL
    const openExplorer = () => {
        // In a real app, would use Linking.openURL
        console.log('Opening explorer:', account.explorerURL);
    };

    // Load balance on component mount
    useEffect(() => {
        fetchBalance();
    }, []);

    return (
        <SafeAreaView style={styles.safeArea}>
            <View style={styles.container}>
                <View style={styles.navBar}>
                    <View style={styles.navBarTitleContainer}>
                        <Text style={styles.navBarTitle}>Account Details</Text>
                    </View>
                    <TouchableOpacity
                        style={styles.logoutButton}
                        onPress={confirmLogout}
                    >
                        <Text style={styles.logoutButtonText}>Logout</Text>
                    </TouchableOpacity>
                </View>

                <ScrollView
                    style={styles.scrollView}
                    contentContainerStyle={styles.scrollViewContent}
                >
                    <View style={styles.sectionContainer}>
                        <Text style={styles.sectionTitle}>Address</Text>

                        <AddressView address={account.address} />

                        <ActionButton
                            title="View on Explorer"
                            icon="safari.fill"
                            style={ButtonStyle.Plain}
                            action={openExplorer}
                        />
                    </View>

                    <View style={styles.sectionContainer}>
                        <Text style={styles.sectionTitle}>Account ID</Text>

                        <AddressView address={account.uniqueAccountId} />
                    </View>

                    <View style={styles.sectionContainer}>
                        <Text style={styles.sectionTitle}>Balance</Text>

                        <View style={styles.balanceContainer}>
                            <Text style={styles.balanceText}>
                                {balance || 'Loading...'}
                            </Text>
                            {isLoadingBalance && (
                                <Text style={styles.loadingText}>Loading...</Text>
                            )}
                        </View>

                        <ActionButton
                            title="Add Funds"
                            progressTitle="Adding Funds..."
                            icon="plus.circle.fill"
                            isLoading={isFunding}
                            style={ButtonStyle.Prominent}
                            action={handleFundAccount}
                        />
                    </View>

                    <ActionButton
                        title="Send Transaction"
                        icon="paperplane.fill"
                        style={ButtonStyle.Prominent}
                        action={() => setShowingSendTransaction(true)}
                    />
                </ScrollView>

                {/* Modal for Send Transaction */}
                <Modal
                    animationType="slide"
                    visible={showingSendTransaction}
                    presentationStyle="pageSheet"
                    onRequestClose={() => setShowingSendTransaction(false)}
                >
                    <SendTransactionView
                        fromAccount={{
                            info: {
                                rpId: account.info.rpId,
                                name: account.info.name,
                                userID: account.info.userID
                            },
                            address: account.address,
                            uniqueAccountId: account.uniqueAccountId,
                        }}
                        onTransactionSent={handleTransactionSent}
                        onDismiss={() => setShowingSendTransaction(false)}
                    />
                </Modal>
            </View>
        </SafeAreaView>
    );
};

const styles = StyleSheet.create({
    safeArea: {
        flex: 1,
        backgroundColor: '#ffffff',
    },
    container: {
        flex: 1,
        backgroundColor: '#ffffff',
    },
    navBar: {
        height: 44,
        flexDirection: 'row',
        alignItems: 'center',
        justifyContent: 'center',
        borderBottomWidth: 1,
        borderBottomColor: '#e0e0e0',
        backgroundColor: '#ffffff',
        paddingHorizontal: 16,
    },
    navBarTitleContainer: {
        flex: 1,
        alignItems: 'center',
    },
    navBarTitle: {
        fontSize: 17,
        fontWeight: '600',
        color: '#000000',
    },
    logoutButton: {
        position: 'absolute',
        right: 16,
    },
    logoutButtonText: {
        fontSize: 17,
        color: '#007AFF',
        fontWeight: '400',
    },
    scrollView: {
        flex: 1,
    },
    scrollViewContent: {
        padding: 20,
    },
    sectionContainer: {
        marginBottom: 24,
    },
    sectionTitle: {
        fontSize: 17,
        fontWeight: '600',
        marginBottom: 12,
        color: '#000000',
    },
    balanceContainer: {
        flexDirection: 'row',
        alignItems: 'center',
        marginBottom: 16,
    },
    balanceText: {
        fontSize: 28,
        fontFamily: 'monospace',
        fontWeight: '500',
        marginRight: 8,
        color: '#000000',
    },
    loadingText: {
        fontSize: 14,
        color: '#888888',
        fontStyle: 'italic',
    },
});

export default AccountDetailsView; 