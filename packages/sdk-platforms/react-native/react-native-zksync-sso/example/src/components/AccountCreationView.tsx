import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity, SafeAreaView } from 'react-native';
import type { AccountInfo, DeployedAccount } from './types';
import PasskeyCreationView from './PasskeyCreationView';

interface AccountCreationViewProps {
    accountInfo: AccountInfo;
    onDeployed: (account: DeployedAccount) => void;
    onDismiss?: () => void;
}

const AccountCreationView: React.FC<AccountCreationViewProps> = ({
    accountInfo,
    onDeployed,
    onDismiss
}) => {
    return (
        <SafeAreaView style={styles.safeArea}>
            <View style={styles.container}>
                <View style={styles.navBar}>
                    <View style={styles.navBarTitleContainer}>
                        <Text style={styles.navBarTitle}>Create Account</Text>
                    </View>
                    <TouchableOpacity
                        style={styles.doneButton}
                        onPress={onDismiss}
                    >
                        <Text style={styles.doneButtonText}>Done</Text>
                    </TouchableOpacity>
                </View>

                <View style={styles.content}>
                    <View style={styles.spacer} />

                    <View style={styles.infoContainer}>
                        <View style={styles.infoRow}>
                            <Text style={styles.infoLabel}>Username:</Text>
                            <View style={styles.spacerFlex} />
                            <Text style={styles.infoValue}>{accountInfo.name}</Text>
                        </View>

                        <View style={styles.infoRow}>
                            <Text style={styles.infoLabel}>User ID:</Text>
                            <View style={styles.spacerFlex} />
                            <Text style={styles.infoMonospace}>{accountInfo.userID}</Text>
                        </View>
                    </View>

                    <PasskeyCreationView
                        accountInfo={accountInfo}
                        onDeployed={onDeployed}
                    />

                    <View style={styles.spacer} />
                </View>
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
    doneButton: {
        position: 'absolute',
        right: 16,
    },
    doneButtonText: {
        fontSize: 17,
        color: '#007AFF',
        fontWeight: '400',
    },
    content: {
        flex: 1,
        padding: 16,
        justifyContent: 'space-between',
    },
    spacer: {
        flex: 1,
    },
    spacerFlex: {
        flex: 1,
    },
    infoContainer: {
        padding: 16,
        backgroundColor: 'rgba(150, 150, 150, 0.1)',
        borderRadius: 12,
        marginBottom: 32,
    },
    infoRow: {
        flexDirection: 'row',
        marginBottom: 16,
        alignItems: 'center',
    },
    infoLabel: {
        fontSize: 16,
        color: '#666666',
    },
    infoValue: {
        fontSize: 16,
        fontWeight: '500',
        color: '#000000',
    },
    infoMonospace: {
        fontSize: 16,
        fontWeight: '500',
        fontFamily: 'monospace',
        color: '#000000',
    },
});

export default AccountCreationView; 