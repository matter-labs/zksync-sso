import React, { useState } from 'react';
import { View, StyleSheet } from 'react-native';
import type { AccountDetails, RpId, RpIdHelper } from './types';
import AccountDetailsView from './AccountDetailsView';
import LoggedOutView from './LoggedOutView';

interface ExampleViewProps {
    rpId: RpId;
}

const ExampleView: React.FC<ExampleViewProps> = ({ rpId }) => {
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
                        name: "JDoe",
                        userID: "jdoe",
                        rpId
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