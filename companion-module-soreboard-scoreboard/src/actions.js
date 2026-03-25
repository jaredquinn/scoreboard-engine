module.exports = {
	GetActionDefinitions(self) {
		const counterChoices = Object.entries(self.widgets || {})
			.filter(([_, w]) => w.type === 'Counter')
			.map(([id, _]) => ({ id: id, label: id }))

		const timerChoices = Object.entries(self.widgets || {})
			.filter(([_, w]) => w.type === 'Timer')
			.map(([id, _]) => ({ id: id, label: id }))

		const listChoices = Object.entries(self.widgets || {})
			.filter(([_, w]) => w.type === 'MappedList')
			.map(([id, _]) => ({ id: id, label: id }))

		const textChoices = Object.entries(self.widgets || {})
			.filter(([_, w]) => w.type === 'StaticText')
			.map(([id, _]) => ({ id: id, label: id }))

		return {
			// --- COUNTER ACTIONS ---
			increment_score: {
				name: 'Counter: Adjust Value',
				options: [
					{ type: 'dropdown', id: 'widget_id', label: 'Counter', default: counterChoices[0]?.id || '', choices: counterChoices },
					{ type: 'number', id: 'amount', label: 'Amount (Negative to decrement)', default: 1 }
				],
				callback: async (event) => {
					const amt = event.options.amount
					await self.sendUpdate(event.options.widget_id, {
						action: amt >= 0 ? 'increment' : 'decrement',
						amount: Math.abs(amt),
					})
				},
			},

			// --- TIMER ACTIONS ---
			timer_control: {
				name: 'Timer: Start/Stop/Reset',
				options: [
					{ type: 'dropdown', id: 'widget_id', label: 'Timer', default: timerChoices[0]?.id || '', choices: timerChoices },
					{ 
						type: 'dropdown', id: 'action', label: 'Action', default: 'start',
						choices: [
							{ id: 'start', label: 'Start' },
							{ id: 'stop', label: 'Stop' },
							{ id: 'reset', label: 'Reset to Initial' }
						] 
					}
				],
				callback: async (event) => {
					await self.sendUpdate(event.options.widget_id, { action: event.options.action })
				},
			},
			timer_adjust: {
				name: 'Timer: Adjust Time (Manual)',
				options: [
					{ type: 'dropdown', id: 'widget_id', label: 'Timer', default: timerChoices[0]?.id || '', choices: timerChoices },
					{ type: 'number', id: 'amount', label: 'Seconds to add (Negative to subtract)', default: 60 }
				],
				callback: async (event) => {
					const amt = event.options.amount
					await self.sendUpdate(event.options.widget_id, {
						action: amt >= 0 ? 'increment' : 'decrement',
						amount: Math.abs(amt),
					})
				},
			},
			timer_settings: {
				name: 'Timer: Set Limits (Admin)',
				options: [
					{ type: 'dropdown', id: 'widget_id', label: 'Timer', default: timerChoices[0]?.id || '', choices: timerChoices },
					{ 
						type: 'dropdown', id: 'action', label: 'Field', default: 'set_initial',
						choices: [
							{ id: 'set_initial', label: 'Initial Seconds' },
							{ id: 'set_min', label: 'Min Seconds' },
							{ id: 'set_max', label: 'Max Seconds' }
						] 
					},
					{ type: 'number', id: 'value', label: 'Value (Seconds)', default: 600 }
				],
				callback: async (event) => {
					await self.sendUpdate(event.options.widget_id, {
						action: event.options.action,
						value: event.options.value,
					})
				},
			},

			// --- LIST ACTIONS ---
			list_nav: {
				name: 'List: Navigation',
				options: [
					{ type: 'dropdown', id: 'widget_id', label: 'List', default: listChoices[0]?.id || '', choices: listChoices },
					{ 
						type: 'dropdown', id: 'action', label: 'Action', default: 'next',
						choices: [
							{ id: 'next', label: 'Next' },
							{ id: 'prev', label: 'Previous' },
							{ id: 'reset', label: 'Reset (Index 0)' }
						] 
					}
				],
				callback: async (event) => {
					await self.sendUpdate(event.options.widget_id, { action: event.options.action })
				},
			},

			// --- STATIC TEXT ACTIONS ---
			update_text: {
				name: 'Text: Update String',
				options: [
					{ type: 'dropdown', id: 'widget_id', label: 'Text Widget', default: textChoices[0]?.id || '', choices: textChoices },
					{ type: 'textinput', id: 'text', label: 'New Text', default: '' }
				],
				callback: async (event) => {
					// Use the 'Value' variant of the update payload for text
					await self.sendUpdate(event.options.widget_id, event.options.text)
				},
			},

			// --- SYSTEM ACTIONS ---
			full_reset: {
				name: 'System: Reset All to Config.xml',
				options: [],
				callback: async () => {
					const url = `http://${self.config.host}:${self.config.port}/widgets/reset`
					await fetch(url, { method: 'POST' })
				},
			}
		}
	},
}

