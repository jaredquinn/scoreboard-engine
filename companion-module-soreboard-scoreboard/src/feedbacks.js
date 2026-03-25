const { combineRgb } = require('@companion-module/base')

module.exports = {
	GetFeedbackDefinitions(self) {
		const timerChoices = Object.entries(self.widgets || {})
			.filter(([_, w]) => w.type === 'Timer')
			.map(([id, _]) => ({ id: id, label: id }))

		const textChoices = Object.entries(self.widgets || {})
			.filter(([_, w]) => w.type === 'StaticText')
			.map(([id, _]) => ({ id: id, label: id }))

		const listChoices = Object.entries(self.widgets || {})
			.filter(([_, w]) => w.type === 'MappedList')
			.map(([id, _]) => ({ id: id, label: id }))

		return {
			// --- TIMER RUNNING FEEDBACK ---
			timer_running: {
				type: 'boolean',
				name: 'Timer: Style When Running',
				defaultStyle: { bgcolor: combineRgb(0, 150, 0), color: combineRgb(255, 255, 255) },
				options: [
					{ type: 'dropdown', id: 'timer_id', label: 'Timer', default: timerChoices[0]?.id || '', choices: timerChoices }
				],
				callback: (feedback) => {
					const isRunning = self.getVariableValue(`${feedback.options.timer_id}_running`)
					return isRunning === 'RUN'
				},
			},

			// --- TEXT MATCH FEEDBACK (NEW) ---
			text_match: {
				type: 'boolean',
				name: 'Text: Style On Match',
				description: 'Change style if the text widget matches a specific string',
				defaultStyle: { bgcolor: combineRgb(255, 255, 0), color: combineRgb(0, 0, 0) }, // Yellow/Black
				options: [
					{ type: 'dropdown', id: 'widget_id', label: 'Text Widget', default: textChoices[0]?.id || '', choices: textChoices },
					{ type: 'textinput', id: 'match_text', label: 'Text to match', default: '' }
				],
				callback: (feedback) => {
					const currentText = self.getVariableValue(feedback.options.widget_id)
					return currentText === feedback.options.match_text
				},
			},

			// --- LIST INDEX FEEDBACK (NEW) ---
			list_index_match: {
				type: 'boolean',
				name: 'List: Style On Specific Item',
				description: 'Change style when a specific item in the list is selected (e.g., FINAL)',
				defaultStyle: { bgcolor: combineRgb(255, 0, 0), color: combineRgb(255, 255, 255) },
				options: [
					{ type: 'dropdown', id: 'widget_id', label: 'List Widget', default: listChoices[0]?.id || '', choices: listChoices },
					{ type: 'textinput', id: 'match_val', label: 'Item Name (e.g. FINAL or OT)', default: '' }
				],
				callback: (feedback) => {
					const currentVal = self.getVariableValue(feedback.options.widget_id)
					return currentVal === feedback.options.match_val
				},
			}
		}
	},
}

