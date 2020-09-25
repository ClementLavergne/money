import {
    get_account_category_amount_by_date,
} from "money"

import {
    clearTableRows
} from "./utils.js"

export { initCategoryTable, refreshCategoryTable }

const addAmoutRow = (table, category_name, category_id) => {
    var row = table.insertRow()
    var category = row.insertCell()
    category.id = "category-" + category_id + "-label"
    category.innerHTML = category_name

    const cellNames = ["expected", "current", "inprogress", "pending"]
    cellNames.forEach(function(amount_name) {
        var cell = row.insertCell()
        cell.id = amount_name + "-" + category_id + "-amount"
        cell.innerHTML = 0.0.toFixed(2) + '€'
    })
}

const initCategoryTable = (type, list) => {
    const table = document.getElementById("overall-" + type)

    // Header
    var header = table.createTHead()
    var row = header.insertRow()
    const titles = [type, "Expected", "Current", "In Progress", "Pending"]
    titles.forEach(element => {
        row.insertCell().innerHTML = element.bold()
    })

    // Remove all rows
    clearTableRows(table)

    // Rows
    list.forEach(function(item) {
        addAmoutRow(table, item, type + '-' + item)
    })

    if (type == "Resource") {
        addAmoutRow(table, "TOTAL", type)
    }
}

const refreshCategoryTable = (account, filter, type, list, categoryTypeId) => {
    var expected = 0.0
    var current = 0.0
    var inProgress = 0.0
    var pending = 0.0
    list.forEach(function(item) {
        const amount = get_account_category_amount_by_date(account, categoryTypeId, item, filter)
        if (amount != undefined) {
            // Update displayed text
            const expectedCell = document.getElementById("expected-" + type + "-" + item + "-amount")
            expectedCell.innerHTML = amount.expected.toFixed(2) + '€'
            if (amount.expected > 0.0) {
                expectedCell.bgColor = 'lightgreen'
            } else if (amount.expected < 0.0) {
                expectedCell.bgColor = 'lightsalmon'
            } else {
                expectedCell.bgColor = 'transparent'
            }

            const currentCell = document.getElementById("current-" + type + "-" + item + "-amount")
            currentCell.innerHTML = amount.current.toFixed(2) + '€'
            if (amount.current > 0.0) {
                currentCell.bgColor = 'lightgreen'
            } else if (amount.current < 0.0) {
                currentCell.bgColor = 'lightsalmon'
            } else {
                currentCell.bgColor = 'transparent'
            }

            const inProgressCell = document.getElementById("inprogress-" + type + "-" + item + "-amount")
            inProgressCell.innerHTML = amount.in_progress.toFixed(2) + '€'
            if (amount.in_progress != 0.0) {
                inProgressCell.bgColor = 'lightsalmon'
            } else {
                inProgressCell.bgColor = 'transparent'
            }

            const pendingCell = document.getElementById("pending-" + type + "-" + item + "-amount")
            pendingCell.innerHTML = amount.pending.toFixed(2) + '€'
            if (amount.pending != 0.0) {
                pendingCell.bgColor = 'lightsalmon'
            } else {
                pendingCell.bgColor = 'transparent'
            }
            // Compute total amount
            expected += amount.expected
            current += amount.current
            inProgress += amount.in_progress
            pending += amount.pending
        }
    })

    if ((list.length > 0) && (type == "Resource")) {
        // Update displayed total
        const expectedCell = document.getElementById("expected-" + type + "-amount")
        expectedCell.innerHTML = expected.toFixed(2) + '€'
        if (expected > 0.0) {
            expectedCell.bgColor = 'lightgreen'
        } else if (expected < 0.0) {
            expectedCell.bgColor = 'lightsalmon'
        } else {
            expectedCell.bgColor = 'transparent'
        }

        const currentCell = document.getElementById("current-" + type + "-amount")
        currentCell.innerHTML = current.toFixed(2) + '€'
        if (current > 0.0) {
            currentCell.bgColor = 'lightgreen'
        } else if (current < 0.0) {
            currentCell.bgColor = 'lightsalmon'
        } else {
            currentCell.bgColor = 'transparent'
        }

        const inProgressCell = document.getElementById("inprogress-" + type + "-amount")
        inProgressCell.innerHTML = inProgress.toFixed(2) + '€'
        if (inProgress != 0.0) {
            inProgressCell.bgColor = 'lightsalmon'
        } else {
            inProgressCell.bgColor = 'transparent'
        }

        const pendingCell = document.getElementById("pending-" + type + "-amount")
        pendingCell.innerHTML = pending.toFixed(2) + '€'
        if (pending != 0.0) {
            pendingCell.bgColor = 'lightsalmon'
        } else {
            pendingCell.bgColor = 'transparent'
        }
    }
}
