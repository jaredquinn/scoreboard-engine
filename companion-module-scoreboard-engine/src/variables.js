module.exports = {
	GetVariableDefinitions(widgets) {
		const defs = [];
		for (const id of Object.keys(widgets)) {
			defs.push({ variableId: id, name: id.replace(/_/g, ' ') });
			if (widgets[id].type === 'Timer') {
				defs.push({ variableId: `${id}_running`, name: `${id} Status` });
				defs.push({ variableId: `${id}_raw`, name: `${id} Raw Value` });
			}
		}
		return defs;
	},

	GetVariableValues(widgets, ss) {
		const values = {};

		for (const [id, w] of Object.entries(widgets)) {
			const { type, data } = w;

			if (type === 'Counter') {
				values[id] = data.value;
			} else if (type === 'Timer') {
				values[`${id}_paused_time`] = data.paused_time;
				values[`${id}_paused`] = data.paused;
				values[`${id}_running`] = data.running ? 'RUN' : 'STOP';
				values[`${id}_raw`] = data.seconds;
				values[id] = data.formatted_time;
			} else if (type === 'List') {
				const { index, options } = data;
				values[id] = options[index] || '---';
			} else if (type === 'Text') {
				values[id] = data.content;
			} else if (type === 'Calculation') {
				values[id] = data.value;
			}
		}
		return values;
	},
}
