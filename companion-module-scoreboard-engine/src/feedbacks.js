const { combineRgb } = require('@companion-module/base')

module.exports = {
	GetFeedbackDefinitions(self) {
		const timerChoices = Object.entries(self.widgets || {})
			.filter(([_, w]) => w.type === 'Timer')
			.map(([id, _]) => ({ id: id, label: id }))

		const textChoices = Object.entries(self.widgets || {})
			.filter(([_, w]) => w.type === 'Text')
			.map(([id, _]) => ({ id: id, label: id }))

		const listChoices = Object.entries(self.widgets || {})
			.filter(([_, w]) => w.type === 'List')
			.map(([id, _]) => ({ id: id, label: id }))

		const switchChoices = Object.entries(self.widgets || {})
			.filter(([_, w]) => w.type === 'Switch')
			.map(([id, _]) => ({ id: id, label: id }))

		const penaltyChoices = Object.entries(self.widgets || {})
			.filter(([_, w]) => w.type === 'PenaltyShots')
			.map(([id, _]) => ({ id: id, label: id }))

		return {
			// --- SWITWCH FEEDBACKS ---
			switch_on: {
				type: 'boolean',
				name: 'Switch Status',
				defaultStyle: { bgcolor: combineRgb(0, 150, 0), color: combineRgb(255, 255, 255) },
				options: [
					{ type: 'dropdown', id: 'switch_id', label: 'Switch', default: switchChoices[0]?.id || '', choices: switchChoices }
				],
				callback: (feedback) => {
					return self.getVariableValue(`${feedback.options.switch_id}`)
				}
			},

			penalty_active_turn: {
				type: 'boolean',
				name: 'Penalties: Active Turn Glow',
				description: 'Highlights the button if this specific round matches the active kicker pointer index',
				defaultStyle: { bgcolor: combineRgb(59, 130, 246), color: combineRgb(255, 255, 255) }, // Blue
				options: [
					{ type: 'dropdown', id: 'widget_id', label: 'Penalty Widget', default: penaltyChoices[0]?.id || '', choices: penaltyChoices },
					{ type: 'number', id: 'target_round', label: 'Target Shot Index (1-5+)', default: 1 }
				],
				callback: (feedback) => {
					const currentRound = self.getVariableValue(`${feedback.options.widget_id}_current_round`)
					// Convert 1-indexed option user input down to 0-indexed engine logic
					return currentRound === (feedback.options.target_round - 1)
				}
			},

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
					return isRunning === true
				},
			},

			timer_paused: {
				type: 'boolean',
				name: 'Timer: Style When Paused',
				defaultStyle: { bgcolor: combineRgb(0, 150, 0), color: combineRgb(255, 255, 255) },
				options: [
					{ type: 'dropdown', id: 'timer_id', label: 'Timer', default: timerChoices[0]?.id || '', choices: timerChoices }
				],
				callback: (feedback) => {
					const isPaused = self.getVariableValue(`${feedback.options.timer_id}_paused`)
					return isPaused === true
				},
			},

			// --- TEXT MATCH FEEDBACK  ---
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

			// --- LIST INDEX FEEDBACK ---
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

