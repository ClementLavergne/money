export {
    clearTableRows,
    getEnumStrings,
    enumStringToIndex,
    enumIndexToString,
    removeChildNodeById,
    removeChildNodesByTagName,
    dateString,
    firstDayCurrentMonthString,
    lastDayCurrentMonthString,
}

const getEnumStrings = (type) => {
    const entries = Object.entries(type)
    const nb_elements = entries.length

    return entries.slice((nb_elements / 2), nb_elements)
}

const enumStringToIndex = (strings, string) => {
    var index = undefined
    for (const entry of strings) {
        if (string == entry[0]) {
            index = entry[1]
            break
        }
    }

    return index
}

const enumIndexToString = (strings, index) => {
    var string = undefined
    for (const entry of strings) {
        if (index == entry[1]) {
            string = entry[0]
            break
        }
    }

    return string
}

const removeChildNodesByTagName  = (node, tagname) => {
    var items = []

    // Get DIV elements
    node.childNodes.forEach(function(item) {
        if (item.tagName == tagname) {
            items.push(item)
        }
    })

    // Remove them
    items.forEach(function(item) {
        node.removeChild(item)
    })
}

const removeChildNodeById  = (node, id) => {
    var items = []

    // Get DIV elements
    node.childNodes.forEach(function(item) {
        if (item.id == id) {
            items.push(item)
        }
    })

    // Remove them
    items.forEach(function(item) {
        node.removeChild(item)
    })
}

const dateString = (date) => {
    var dd = String(date.getDate()).padStart(2, '0');
    var mm = String(date.getMonth() + 1).padStart(2, '0');
    var yyyy = date.getFullYear();

    return yyyy + '-' + mm + '-' + dd;
}

const firstDayCurrentMonthString = () => {
    const today = new Date()
    const firstDay = new Date(today.getFullYear(), today.getMonth(), 1)

    return dateString(firstDay)
}

const lastDayCurrentMonthString = () => {
    var today = new Date()
    const lastDay = new Date(today.getFullYear(), today.getMonth() + 1, 0)

    return dateString(lastDay)
}

const clearTableRows = (table) => {
    for (var i=table.rows.length-1; i >=1; i--) {
        table.deleteRow(i)
    }
}