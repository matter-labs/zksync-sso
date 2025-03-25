import React from 'react';
import { TouchableOpacity, Text, StyleSheet, ActivityIndicator, View } from 'react-native';

export enum ButtonStyle {
    Prominent = 'prominent',
    Destructive = 'destructive',
    Plain = 'plain'
}

interface ActionButtonProps {
    title: string;
    progressTitle?: string;
    icon?: string;
    isLoading?: boolean;
    isDisabled?: boolean;
    style?: ButtonStyle;
    action: () => void;
}

const ActionButton: React.FC<ActionButtonProps> = ({
    title,
    progressTitle,
    icon,
    isLoading = false,
    isDisabled = false,
    style = ButtonStyle.Prominent,
    action
}) => {
    const getButtonStyle = () => {
        switch (style) {
            case ButtonStyle.Prominent:
                return styles.prominentButton;
            case ButtonStyle.Destructive:
                return styles.destructiveButton;
            case ButtonStyle.Plain:
                return styles.plainButton;
            default:
                return styles.prominentButton;
        }
    };

    const getTextStyle = () => {
        switch (style) {
            case ButtonStyle.Prominent:
                return styles.prominentButtonText;
            case ButtonStyle.Destructive:
                return styles.destructiveButtonText;
            case ButtonStyle.Plain:
                return styles.plainButtonText;
            default:
                return styles.prominentButtonText;
        }
    };

    const getLoaderColor = () => {
        switch (style) {
            case ButtonStyle.Prominent:
                return '#ffffff';
            case ButtonStyle.Destructive:
                return '#FF3B30';
            case ButtonStyle.Plain:
                return '#007AFF';
            default:
                return '#ffffff';
        }
    };

    const renderIcon = () => {
        if (!icon) return null;

        const getIconContent = () => {
            switch (icon) {
                case 'plus':
                    return '+';
                case 'person':
                    return '👤';
                case 'paperplane':
                    return '▶️';
                case 'safari':
                    return '🔍';
                case 'plus.circle.fill':
                    return '+';
                case 'paperplane.fill':
                    return '→';
                case 'safari.fill':
                    return '🔍';
                default:
                    return '•';
            }
        };

        return (
            <View style={[
                styles.iconContainer,
                style === ButtonStyle.Prominent ? styles.prominentIcon : styles.plainIcon
            ]}>
                <Text style={[
                    styles.iconText,
                    style === ButtonStyle.Prominent ? styles.prominentIconText : styles.plainIconText
                ]}>
                    {getIconContent()}
                </Text>
            </View>
        );
    };

    return (
        <View style={styles.sectionContainer}>
            <TouchableOpacity
                style={[
                    styles.actionButton,
                    getButtonStyle(),
                    (isDisabled || isLoading) && styles.disabledButton
                ]}
                onPress={action}
                disabled={isDisabled || isLoading}
            >
                <View style={styles.buttonContent}>
                    {isLoading && (
                        <ActivityIndicator
                            size="small"
                            color={getLoaderColor()}
                            style={styles.loadingIndicator}
                        />
                    )}

                    {!isLoading && renderIcon()}

                    <Text style={[
                        styles.actionButtonText,
                        getTextStyle()
                    ]}>
                        {isLoading && progressTitle ? progressTitle : title}
                    </Text>
                </View>
            </TouchableOpacity>
        </View>
    );
};

const styles = StyleSheet.create({
    sectionContainer: {
        marginVertical: 0,
        paddingHorizontal: 0,
    },
    actionButton: {
        borderRadius: 8,
        height: 50,
        justifyContent: 'center',
        overflow: 'hidden',
    },
    buttonContent: {
        flexDirection: 'row',
        alignItems: 'center',
        justifyContent: 'center',
        paddingHorizontal: 16,
    },
    loadingIndicator: {
        marginRight: 8,
    },
    plainButton: {
        backgroundColor: '#F2F2F7',
    },
    prominentButton: {
        backgroundColor: '#2979FF',
    },
    destructiveButton: {
        backgroundColor: 'transparent',
    },
    disabledButton: {
        opacity: 0.6,
    },
    actionButtonText: {
        fontSize: 17,
        fontWeight: '600',
        textAlign: 'center',
    },
    plainButtonText: {
        color: '#2979FF',
    },
    prominentButtonText: {
        color: 'white',
    },
    destructiveButtonText: {
        color: '#FF3B30',
    },
    iconContainer: {
        width: 22,
        height: 22,
        justifyContent: 'center',
        alignItems: 'center',
        marginRight: 8,
    },
    iconText: {
        fontSize: 18,
        fontWeight: 'bold',
    },
    prominentIcon: {
        backgroundColor: 'transparent',
    },
    plainIcon: {
        backgroundColor: 'transparent',
    },
    prominentIconText: {
        color: 'white',
    },
    plainIconText: {
        color: '#2979FF',
    },
});

export default ActionButton; 