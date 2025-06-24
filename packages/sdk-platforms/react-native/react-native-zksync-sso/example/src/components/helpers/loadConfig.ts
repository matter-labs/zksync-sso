import { Platform } from 'react-native';
import { type Config } from '../../../../src';

/**
 * Loads configuration from a JSON file bundled with the app
 * Automatically handles localhost URLs for Android emulator by replacing with 10.0.2.2
 * @returns Configuration object with platform-specific URL adjustments
 */
export const loadConfig = (): Config => {
    const config: Config = require('../../config.json');
    console.log('Successfully loaded config from bundled JSON');

    // Handle localhost URLs for Android emulator
    if (Platform.OS === 'android' && config.nodeUrl.includes('localhost')) {
        const originalUrl = config.nodeUrl;
        config.nodeUrl = config.nodeUrl.replace('localhost', '10.0.2.2');
        console.log(`Android: Replaced localhost URL "${originalUrl}" with "${config.nodeUrl}"`);
    } else {
        console.log(`${Platform.OS}: Using original nodeUrl "${config.nodeUrl}"`);
    }

    return config;
};