import { type Config } from '../../../../src';

/**
 * Loads configuration from a JSON file bundled with the app
 * @returns Configuration object
 */
export const loadConfig = (): Config => {
    const config = require('../../config.json');
    console.log('Successfully loaded config from bundled JSON');
    return config;
};