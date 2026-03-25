const { Regex } = require('@companion-module/base')

module.exports = {
	GetConfigFields() {
		return [
			{
				type: 'static-text',
				id: 'info',
				width: 12,
				label: 'Information',
				value: 'This module connects to the Rust Scoreboard Engine API. Ensure your server is running and accessible.',
			},
			{
				type: 'textinput',
				id: 'host',
				label: 'Target IP / Hostname',
				width: 8,
				default: '127.0.0.1',
			},
			{
				type: 'textinput',
				id: 'port',
				label: 'Target Port',
				width: 4,
				default: '3000',
				regex: Regex.PORT,
			},
			{
				type: 'checkbox',
				id: 'verbose',
				label: 'Enable Verbose Logging',
				default: false,
				width: 12,
			},
		]
	},
}
