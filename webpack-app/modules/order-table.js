import {
    get_account_categories,
    set_account_order_date,
    set_account_order_description,
    set_account_order_amount,
    set_account_order_resource,
    set_account_order_tags,
    set_account_order_state,
    delete_account_order,
    toggle_account_order_visibility,
    get_account_filtered_orders,
    TransactionState,
    CategoryType,
} from "money"

import {
    dateString,
    enumStringToIndex,
    clearTableRows,
    getEnumStrings,
} from "./utils.js"

export { initOrderTable }

// Enumerations
const transactionStateEnum = getEnumStrings(TransactionState)
const categoryTypeEnum = getEnumStrings(CategoryType)
const resourceCategoryType = enumStringToIndex(categoryTypeEnum, "Resource")
const tagCategoryType = enumStringToIndex(categoryTypeEnum, "Tag")
// Node
const ordersTable = document.getElementById("orders")

const addOrderRow = (order, account, render_func) => {
    const row = ordersTable.insertRow()
    const resourceList = get_account_categories(account, resourceCategoryType)
    const tagList = get_account_categories(account, tagCategoryType)

    // Date
    var date = document.createElement("input")
    date.type = "text"
    date.value = order.order.date
    date.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (set_account_order_date(account, order.id, date.value)) {
                console.log("Order " + order.id + " date: " + date.value)
                requestAnimationFrame(render_func)
            } else {
                alert("Expected date format:  YY-MM-DD")
            }
        }
    })
    date.addEventListener('click', () => {
        date.value = dateString(new Date())
    })
    row.insertCell().appendChild(date)

    // Description node
    var description = document.createElement("input")
    description.type = "text"
    description.value = order.order.description
    description.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (set_account_order_description(account, order.id, description.value)) {
                console.log("Order " + order.id + " description: " + description.value)
                requestAnimationFrame(render_func)
            }
        }
    })
    row.insertCell().appendChild(description)

    // Amount node
    var amount = document.createElement("input")
    amount.type = "text"
    amount.value = order.order.amount.toFixed(2) + "€"
    amount.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (amount.value == "") {
                amount.value = "0.0"
            }

            const float = parseFloat(amount.value)
            if (set_account_order_amount(account, order.id, float)) {
                console.log("Order " + order.id + " amount: " + float.toFixed(2))
                requestAnimationFrame(render_func)
            }
        }
    })
    amount.addEventListener('click', event => {
        amount.value = ""
    })
    row.insertCell().appendChild(amount)

    // Resource node
    var resource = document.createElement("select")
    var empty_option = document.createElement("option")
    empty_option.value = "-"
    empty_option.text = "-"
    empty_option.disabled = true
    resource.appendChild(empty_option)
    resourceList.forEach(function(item) {
        var option = document.createElement("option")
        option.value = item
        option.text = item
        resource.appendChild(option)
    })
    if (order.order.resource != null) {
        resource.value = order.order.resource
    } else {
        resource.value = "-"
    }
    resource.addEventListener('change', function() {
        if (set_account_order_resource(account, order.id, this.value)) {
            console.log("Order " + order.id + " resource: " + this.value)
            requestAnimationFrame(render_func)
        }
    }, false)
    row.insertCell().appendChild(resource)

    // Tags
    var tags = document.createElement("select")
    tags.multiple = true
    tagList.forEach(function(item) {
        var option = document.createElement("option")
        option.value = item
        option.text = item

        order.order.tags.forEach(function(tag) {
            if (item == tag) {
                option.selected = true
            }
        })

        tags.appendChild(option)
    })
    tags.addEventListener('change', function() {
        const selectedValues = [...this.options]
                     .filter((x) => x.selected)
                     .map((x)=>x.value)

        if (set_account_order_tags(account, order.id, selectedValues)) {
            console.log("Order " + order.id + " tags: " + selectedValues)
            requestAnimationFrame(render_func)
        } else {
            console.error("Unknown ", order.id, selectedValues)
        }
    }, false)
    row.insertCell().appendChild(tags)

    // State
    var state = document.createElement("select")
    for (const entry of transactionStateEnum) {
        var option = document.createElement("option")
        option.text = entry[0]
        option.value = entry[0]
        state.appendChild(option)
    }
    state.value = order.order.state
    state.addEventListener('change', function() {
        const index = enumStringToIndex(transactionStateEnum, this.value)
        if (set_account_order_state(account, order.id, index)) {
            console.log("Order " + order.id + " state: " + this.value)
            requestAnimationFrame(render_func)
        }
    }, false)
    row.insertCell().appendChild(state)

    // Remove button node
    if (order.order.visible) {
        var remove_button = document.createElement("input")
        remove_button.type = "button"
        remove_button.value = "remove"
        remove_button.addEventListener('click', event => {
            if (toggle_account_order_visibility(account, order.id)) {
                console.log("Order " + order.id + ": removed!")
            }
            requestAnimationFrame(render_func)
        })
        row.insertCell().appendChild(remove_button)
    } else {
        var restore_button = document.createElement("input")
        var delete_button = document.createElement("input")
        restore_button.type = "button"
        restore_button.value = "restore"
        delete_button.type = "button"
        delete_button.value = "delete"
        restore_button.addEventListener('click', event => {
            if (toggle_account_order_visibility(account, order.id)) {
                console.log("Order " + order.id + ": restored!")
            }
            requestAnimationFrame(render_func)
        })
        delete_button.addEventListener('click', event => {
            if (confirm("Are you sure?")) {
                if (delete_account_order(account, order.id)) {
                    console.log("Order " + order.id + ": deleted!")
                }
                requestAnimationFrame(render_func)
            }
        })
        row.insertCell().appendChild(restore_button)
        row.insertCell().appendChild(delete_button)
    }
}

const initOrderTable = (account, filter, render_func) => {
    const orders = get_account_filtered_orders(account, filter)

    if (!orders.length == 0) {
        // Header
        var header = ordersTable.createTHead()
        var row = header.insertRow()
        const titles = ["Date", "Description", "Amount", "Resource", "Tags", "State", "", ""]
        titles.forEach(element => {
            row.insertCell().innerHTML = element.bold()
        })

        // Remove all rows
        clearTableRows(ordersTable)

        // Rows
        orders.forEach(function(item) {
            addOrderRow(JSON.parse(item), account, render_func)
        })
    } else {
        // Remove all rows
        clearTableRows(ordersTable)
    }
}
