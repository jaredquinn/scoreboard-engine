module.exports = {
	GetVariableDefinitions(widgets) {
		const defs = [];
		for (const id of Object.keys(widgets)) {
			defs.push({ variableId: id, name: id.replace(/_/g, ' ') });
			if (widgets[id].type === 'Timer') {
				defs.push({ variableId: `${id}_running`, name: `${id} Status` });
			}
		}
		return defs;
	},

	GetVariableValues(widgets) {
		const values = {};
		for (const [id, w] of Object.entries(widgets)) {
			const { type, data } = w;

			if (type === 'Counter') {
				values[id] = data.value;
			} else if (type === 'Timer') {
				const timeStr = new Date(data.seconds * 1000).toISOString().substr(11, 8);
				values[id] = timeStr;
				values[`${id}_running`] = data.running ? 'RUN' : 'STOP';
				values[`${id}_formatted`] = data.formatted_time;
			} else if (type === 'MappedList') {
				const [idx, options] = data;
				values[id] = options[idx] || '---';
			} else if (type === 'StaticText') {
				values[id] = data;
			}
		}
		return values;
	},
}
