import React, { useState } from 'react';
import { View, StyleSheet } from 'react-native';
import type { AccountDetails } from './types';
import AccountDetailsView from './AccountDetailsView';
import LoggedOutView from './LoggedOutView';

interface ExampleViewProps {
    relyingPartyIdentifier: string;
}

const ExampleView: React.FC<ExampleViewProps> = ({ relyingPartyIdentifier }) => {
    const [accountDetails, setAccountDetails] = useState<AccountDetails | null>(null);

    return (
        <View style={styles.navigationContainer}>
            {accountDetails ? (
                <AccountDetailsView
                    account={accountDetails}
                    onLogout={() => {
                        setAccountDetails(null);
                    }}
                />
            ) : (
                <LoggedOutView
                    accountInfo={{
                        name: "Jane Doe",
                        userID: "jdoe@example.com",
                        domain: relyingPartyIdentifier,
                    }}
                    onAccountCreated={(account) => {
                        setAccountDetails(account);
                    }}
                    onSignedIn={(account) => {
                        setAccountDetails(account);
                    }}
                />
            )}
        </View>
    );
};

const styles = StyleSheet.create({
    navigationContainer: {
        flex: 1,
        width: '100%',
    },
});

export default ExampleView; 