import React, { useState } from 'react';
import { View, Text, StyleSheet, Modal, SafeAreaView } from 'react-native';
import { type AccountInfo, type AccountDetails, createAccountDetails } from './types';
import ActionButton, { ButtonStyle } from './ActionButton';
import AccountCreationView from './AccountCreationView';
import { getAccountByUserIdWrapper } from './helpers/getAccountByUserId';

interface LoggedOutViewProps {
    accountInfo: AccountInfo;
    onAccountCreated: (account: AccountDetails) => void;
    onSignedIn: (account: AccountDetails) => void;
}

const LoggedOutView: React.FC<LoggedOutViewProps> = ({
    accountInfo,
    onAccountCreated,
    onSignedIn
}) => {
    const [showingCreateAccount, setShowingCreateAccount] = useState(false);
    const [isSigningIn, setIsSigningIn] = useState(false);

    const handleSignIn = async () => {
        setIsSigningIn(true);

        try {
            const uniqueAccountId = accountInfo.userID;

            const deployedAccount = await getAccountByUserIdWrapper(
                uniqueAccountId
            );

            const accountDetails = createAccountDetails(accountInfo, deployedAccount);

            console.log('Signed in with account:', accountDetails);
            onSignedIn(accountDetails);
        } catch (error) {
            console.log('error: ', error);
        } finally {
            setIsSigningIn(false);
        }
    };

    return (
        <SafeAreaView style={styles.safeArea}>
            <View style={styles.container}>
                <View style={styles.contentWrapper}>
                    <View style={styles.iconAndTitleContainer}>
                        <View style={styles.iconPlaceholder}>
                            <Text style={styles.iconText}>🔑</Text>
                        </View>

                        <View style={styles.titleContainer}>
                            <Text style={styles.title}>ZKsync SSO Example</Text>
                            <Text style={styles.subtitle}>Create an account or sign in with passkeys</Text>
                        </View>
                    </View>

                    <View style={styles.buttonsContainer}>
                        <ActionButton
                            title="Create Account"
                            icon="plus"
                            style={ButtonStyle.Prominent}
                            action={() => setShowingCreateAccount(true)}
                        />

                        <View style={styles.buttonSpacing} />

                        <ActionButton
                            title="Sign In"
                            icon="person"
                            progressTitle="Signing In..."
                            isLoading={isSigningIn}
                            style={ButtonStyle.Plain}
                            action={handleSignIn}
                        />
                    </View>
                </View>
            </View>

            {showingCreateAccount && (
                <Modal
                    animationType="slide"
                    visible={showingCreateAccount}
                    presentationStyle="pageSheet"
                    onRequestClose={() => setShowingCreateAccount(false)}
                >
                    <AccountCreationView
                        accountInfo={accountInfo}
                        onDeployed={(deployedAccount) => {
                            setShowingCreateAccount(false);
                            if (onAccountCreated) {
                                const accountDetails = createAccountDetails(accountInfo, deployedAccount);
                                onAccountCreated(accountDetails);
                            }
                        }}
                        onDismiss={() => setShowingCreateAccount(false)}
                    />
                </Modal>
            )}
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
        paddingHorizontal: 24,
    },
    contentWrapper: {
        flex: 1,
        justifyContent: 'center',
        alignItems: 'stretch',
        paddingBottom: 60,
    },
    iconAndTitleContainer: {
        alignItems: 'center',
        marginBottom: 60,
    },
    iconPlaceholder: {
        width: 80,
        height: 80,
        borderRadius: 40,
        backgroundColor: '#2979FF20',
        justifyContent: 'center',
        alignItems: 'center',
        marginBottom: 24,
    },
    iconText: {
        fontSize: 40,
    },
    titleContainer: {
        alignItems: 'center',
    },
    title: {
        fontSize: 24,
        fontWeight: 'bold',
        marginBottom: 8,
        textAlign: 'center',
        color: '#000000',
    },
    subtitle: {
        fontSize: 16,
        color: '#666666',
        textAlign: 'center',
        marginHorizontal: 20,
    },
    buttonsContainer: {
        alignItems: 'stretch',
        paddingHorizontal: 12,
    },
    buttonSpacing: {
        height: 16,
    },
});

export default LoggedOutView; 