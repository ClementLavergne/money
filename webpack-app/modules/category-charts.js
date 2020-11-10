import Chart from 'chart.js';

import {
    CategoryType,
    get_account_absolute_category_amount_by_date,
    get_account_relative_category_amount_by_date,
    get_account_categories,
} from "money"

import {
    lastDayCurrentMonthString,
    lastDayPreviousMonthString,
    enumStringToIndex,
    getEnumStrings,
} from "./utils.js"

export {
    refreshBalanceChart,
    refreshResourceChart,
    refreshTagChart,
    refreshTransactionChart,
    clearBalanceChart,
    clearResourceChart,
    clearTagChart,
    clearTransactionChart,
}

// Enumerations
const categoryTypeEnum = getEnumStrings(CategoryType)
const resourceCategoryType = enumStringToIndex(categoryTypeEnum, "Resource")
const tagCategoryType = enumStringToIndex(categoryTypeEnum, "Tag")
// Charts
const balanceCtx = document.getElementById('balanceChart');
var balanceChart = null
const resourceCtx = document.getElementById('resourceChart');
var resourceChart = null
const tagCtx = document.getElementById('tagChart');
var tagChart = null
const transactionCtx = document.getElementById('transactionChart');
var transactionChart = null
// Dataset colors
const colorBank = [
    [255, 99, 132],
    [65, 186, 255],
    [128, 0, 128],
    [205, 92, 92],
    [0, 128, 128],
    [0, 0, 128],
    [255, 160, 122],
    [0, 0, 0],
    [128, 128, 0],
    [233, 150, 122],
    [0, 255, 255],
    [240, 128, 128],
    [250, 128, 114],
    [255, 0, 0],
    [128,0, 0],
    [0, 255, 0],
    [0, 0, 255],
    [255, 0, 255],
]
var colorIndex = 0

const resetColor = () => {
    colorIndex = 0
}

const nextColor = () => {
    if (colorIndex < (colorBank.length - 1)) {
        colorIndex++
    } else {
        colorIndex = 0
    }
}

const currentRgbaColor = (opacity) => {
    return "rgba(" + colorBank[colorIndex][0] + ',' + colorBank[colorIndex][1] + ',' + colorBank[colorIndex][2] + ',' + opacity + ')'
}

// Should be launched once
const initBalanceChart = () => {
    balanceChart = new Chart(balanceCtx, {
        type: 'bar',
        data: {
            datasets: []
        },
        options: {
            title: {
                text: "Balance and Gain",
                display: true
            },
            scales: {
                xAxes: [{
                    type: 'time',
                    distribution: 'series',
                    time: {
                        unit: 'month',
                        minUnit: 'day',
                        tooltipFormat: 'll'
                    }
                }],
                yAxes: [{
                    id: 'A',
                    stacked: false,
                    ticks: {
                        beginAtZero: true,
                        callback: function(value, index, values) {
                            return value + '€';
                        }
                    }
                },
                {
                    id: 'B',
                    position: 'right',
                    stacked: false,
                    ticks: {
                        beginAtZero: true,
                        callback: function(value, index, values) {
                            return value + '%';
                        }
                    }
                },
            ]
            },
            tooltips: {
                callbacks: {
                    title: function(tooltipItems, data) {
                        return '[' + data.datasets[tooltipItems[0].datasetIndex].start[tooltipItems[0].index].format('ll') + '; ' + tooltipItems[0].xLabel + ']';
                    },
                    label: function(tooltipItem, data) {
                        if (data.datasets[tooltipItem.datasetIndex].yAxisID == 'A') {
                            return tooltipItem.yLabel + '€'
                        } else {
                            return tooltipItem.yLabel + '%'
                        }
                    }
                }
            }
        }
    });

    resetColor()
    balanceChart.data.datasets.push({
        label: "Monthly balance",
        start: [],
        data: [],
        borderWidth: 3,
        backgroundColor: currentRgbaColor(0.4),
        borderColor: currentRgbaColor(1),
        yAxisID: 'A'
    })
    nextColor()
    balanceChart.data.datasets.push({
        label: "Monthly resource gain",
        start: [],
        type: 'line',
        data: [],
        borderWidth: 3,
        backgroundColor: currentRgbaColor(0.2),
        borderColor: currentRgbaColor(1),
        yAxisID: 'B'
    })
}

const clearBalanceChart = () => {
    if (balanceChart != null) {
        balanceChart.destroy()
        balanceChart = null
    }
}

const refreshBalanceChart = (days, account) => {
    const resourceList = get_account_categories(account, resourceCategoryType)
    const lastDayPreviousMonth = lastDayPreviousMonthString(new Date(days[0]))
    const allDays = [lastDayPreviousMonth, ...days]
    var totalAbsolutePrev = undefined

    // Initialize if necessary
    if (balanceChart == null) {
        initBalanceChart()
    }

    // Resets old data
    balanceChart.data.datasets.forEach(function(dataset) {
        dataset.start = []
        dataset.data = []
    })
    allDays.forEach(function(firstDay, index) {
        const moment = require('moment')
        const convertedFirstDay = moment(firstDay)
        const lastDayMonth = lastDayCurrentMonthString(new Date(firstDay))
        const convertedLastDayMonth = new Date(lastDayMonth)
        var arrayRelative = []
        var arrayAbsolute = []
        resourceList.forEach(function(item) {
            if (index > 0) {
                const relativeData = get_account_relative_category_amount_by_date(account, resourceCategoryType, item, firstDay, lastDayMonth)
                if (relativeData != undefined) {
                    arrayRelative.push(relativeData)
                }
            }
            const absoluteData = get_account_absolute_category_amount_by_date(account, resourceCategoryType, item, lastDayMonth)
            if (absoluteData != undefined) {
                arrayAbsolute.push(absoluteData)
            }
        })

        // Balance
        if (index > 0) {
            if (arrayRelative.length > 0) {
                balanceChart.data.datasets[0].start.push(convertedFirstDay)
                balanceChart.data.datasets[0].data.push({
                    t: convertedLastDayMonth,
                    y: arrayRelative.reduce((acc, value) => acc + value.expected, 0.0).toFixed(2)
                })
            }
        }

        // Gain
        if (arrayAbsolute.length > 0) {
            const totalAbsolute = arrayAbsolute.reduce((acc, value) => acc + value.expected, 0.0)
            if (totalAbsolutePrev != undefined) {
                balanceChart.data.datasets[1].start.push(convertedFirstDay)
                balanceChart.data.datasets[1].data.push({
                    t: convertedLastDayMonth,
                    y: (((totalAbsolute / totalAbsolutePrev) - 1.0) * 100.0).toFixed(2)
                })
            } else {
                console.log("Unable to compute Gain at " + lastDayMonth)
            }

            totalAbsolutePrev = totalAbsolute
        }
    })

    balanceChart.update()
}

const initResourceChart = (resourceList) => {
    resourceChart = new Chart(resourceCtx, {
        type: 'bar',
        data: {
            datasets: []
        },
        options: {
            title: {
                text: "Absolute resource results",
                display: true
            },
            elements: {
                line: {
                    tension: 0,
                    stepped: false
                }
            },
            scales: {
                xAxes: [{
                    type: 'time',
                    distribution: 'series',
                    time: {
                        unit: 'day',
                        minUnit: 'day',
                        tooltipFormat: 'll'
                    }
                }],
                yAxes: [{
                    stacked: false,
                    ticks: {
                        beginAtZero: true,
                        callback: function(value, index, values) {
                            return value + '€'
                        }
                    }
                }]
            },
            tooltips: {
                callbacks: {
                    label: function(tooltipItem, data) {
                        return tooltipItem.yLabel + '€'
                    },
                    beforeLabel: function(tooltipItem, data) {
                        const labels = data.datasets[tooltipItem.datasetIndex].labels

                        if (labels != undefined) {
                            return labels[tooltipItem.index]
                        } else {
                            return null
                        }
                    }
                }
            }
        }
    });

    resetColor()
    resourceChart.data.datasets.push({
        type: 'line',
        label: "TOTAL",
        data: [],
        borderWidth: 3,
        borderColor: currentRgbaColor(1),
        backgroundColor: currentRgbaColor(0.2),
        pointStyle: 'triangle'
    })
    resourceList.forEach(function(resource) {
        nextColor()
        resourceChart.data.datasets.push({
            type: 'line',
            label: resource,
            labels: [],
            data: [],
            borderWidth: 1,
            borderColor: currentRgbaColor(1),
            backgroundColor: currentRgbaColor(0.1),
            pointStyle: 'circle',
            hidden: true
        })
    })
}

const clearResourceChart = () => {
    if (resourceChart != null) {
        resourceChart.destroy()
        resourceChart = null
    }
}

const refreshResourceChart = (filteredPlots, sortedDates, account) => {
    const resourceList = get_account_categories(account, resourceCategoryType)
    const validDates = sortedDates.filter(date => date != null)
    const nbDates = validDates.length
    var total = []

    // Initialize if necessary
    if (resourceChart == null) {
        initResourceChart(resourceList)
    }

    // Reset old data
    resourceChart.data.datasets.forEach(function(dataset) {
        dataset.start = []
        dataset.data = []
        dataset.labels = []
    })
    // Initialize total amount
    resourceList.forEach(function() {
        total.push(0.0)
    })
    validDates.forEach(function(date, date_index) {
        const converted_date = new Date(date)
        const values = filteredPlots.get(date)

        // Absolute amount at date
        resourceList.forEach(function(resource, resource_index) {
            var addPlot = false
            values.forEach(function(value) {
                if ((value.resource == resource) || (date_index == 0) || (date_index == nbDates - 1)) {
                    addPlot = true
                }
            })

            if (addPlot) {
                var absoluteResource = get_account_absolute_category_amount_by_date(account, resourceCategoryType, resource, date)
                // Discard undefined result
                if (absoluteResource == undefined) {
                    absoluteResource = 0.0
                } else {
                    absoluteResource = absoluteResource.expected
                }

                if (absoluteResource != undefined) {
                    // Add plot to specific resource
                    resourceChart.data.datasets[resource_index + 1].data.push({
                        t: converted_date,
                        y: absoluteResource.toFixed(2)
                    })
                    // Add labels to first plot if available
                    resourceChart.data.datasets[resource_index + 1].labels.push(values.filter(value => value.resource === resource).map(value => value.description).join(', '))

                    // Update total amount at this date
                    total[resource_index] = absoluteResource
                }
            }
        })

        // Add plot to total amounts
        resourceChart.data.datasets[0].data.push({
            t: converted_date,
            y: total.reduce((acc, absolute) => acc + absolute, 0.0).toFixed(2)
        })
    })

    resourceChart.update()
}

const initTagChart = (tagList) => {
    tagChart = new Chart(tagCtx, {
        type: 'bar',
        data: {
            datasets: []
        },
        options: {
            title: {
                text: "Absolute tag results",
                display: true
            },
            elements: {
                line: {
                    tension: 0,
                    stepped: false
                }
            },
            scales: {
                xAxes: [{
                    type: 'time',
                    distribution: 'series',
                    time: {
                        unit: 'day',
                        minUnit: 'day',
                        tooltipFormat: 'll'
                    }
                }],
                yAxes: [{
                    stacked: false,
                    ticks: {
                        beginAtZero: true,
                        callback: function(value, index, values) {
                            return value + '€'
                        }
                    }
                }]
            },
            tooltips: {
                callbacks: {
                    label: function(tooltipItem, data) {
                        return tooltipItem.yLabel + '€'
                    },
                    beforeLabel: function(tooltipItem, data) {
                        const labels = data.datasets[tooltipItem.datasetIndex].labels

                        if (labels != undefined) {
                            return labels[tooltipItem.index]
                        } else {
                            return null
                        }
                    }
                }
            }
        }
    });

    resetColor()
    tagList.forEach(function(tag) {
        nextColor()
        tagChart.data.datasets.push({
            type: 'line',
            label: tag,
            labels: [],
            data: [],
            borderWidth: 1,
            backgroundColor: currentRgbaColor(0.2),
            borderColor: currentRgbaColor(1),
            pointStyle: 'circle',
            hidden: true
        })
    })
}

const clearTagChart = () => {
    if (tagChart != null) {
        tagChart.destroy()
        tagChart = null
    }
}

const refreshTagChart = (filteredPlots, sortedDates, account) => {
    const tagList = get_account_categories(account, tagCategoryType)
    const validDates = sortedDates.filter(date => date != null)
    const nbDates = validDates.length

    // Initialize if necessary
    if (tagChart == null) {
        initTagChart(tagList)
    }

    // Reset old data
    tagChart.data.datasets.forEach(function(dataset) {
        dataset.start = []
        dataset.data = []
        dataset.labels = []
    })
    validDates.forEach(function(date, date_index) {
        const converted_date = new Date(date)
        const values = filteredPlots.get(date)

        // Absolute amount at date
        tagList.forEach(function(tag, tag_index) {
            var addPlot = false
            values.forEach(function(value) {
                if ((value.tags.includes(tag)) || (date_index == 0) || (date_index == nbDates - 1)) {
                    addPlot = true
                }
            })

            if (addPlot) {
                var absoluteResource = get_account_absolute_category_amount_by_date(account, tagCategoryType, tag, date)
                // Discard undefined result
                if (absoluteResource == undefined) {
                    absoluteResource = 0.0
                } else {
                    absoluteResource = absoluteResource.expected
                }

                // Add plot to specific tag
                tagChart.data.datasets[tag_index].data.push({
                    t: converted_date,
                    y: absoluteResource.toFixed(2)
                })

                // Add labels to first plot if available
                tagChart.data.datasets[tag_index].labels.push(values.filter(value => value.tags.includes(tag)).map(value => value.description).join(', '))
            }
        })
    })

    tagChart.update()
}

const initTransactionChart = () => {
    transactionChart = new Chart(transactionCtx, {
        type: 'bar',
        data: {
            datasets: []
        },
        options: {
            title: {
                text: "Displayed transactions",
                display: true
            },
            elements: {
                line: {
                    tension: 0,
                    stepped: false
                }
            },
            scales: {
                xAxes: [{
                    type: 'time',
                    distribution: 'series',
                    time: {
                        unit: 'day',
                        minUnit: 'day',
                        tooltipFormat: 'll'
                    }
                }],
                yAxes: [{
                    stacked: false,
                    ticks: {
                        beginAtZero: true,
                        callback: function(value, index, values) {
                            return value + '€'
                        }
                    }
                }]
            },
            tooltips: {
                callbacks: {
                    label: function(tooltipItem, data) {
                        return tooltipItem.yLabel + '€'
                    },
                    beforeLabel: function(tooltipItem, data) {
                        const labels = data.datasets[tooltipItem.datasetIndex].labels

                        if (labels != undefined) {
                            return labels[tooltipItem.index]
                        } else {
                            return null
                        }
                    }
                }
            }
        }
    })

    resetColor()
    transactionChart.data.datasets.push({
        label: "Daily amount",
        labels: [],
        data: [],
        borderWidth: 2,
        backgroundColor: currentRgbaColor(1),
        borderColor: currentRgbaColor(1)
    })
}

const clearTransactionChart = () => {
    if (transactionChart != null) {
        transactionChart.destroy()
        transactionChart = null
    }
}

const refreshTransactionChart = (filteredPlots, sortedDates) => {
    // Initialize if necessary
    if (transactionChart == null) {
        initTransactionChart()
    }

    // Reset old data
    transactionChart.data.datasets.forEach(function(dataset) {
        dataset.start = []
        dataset.data = []
        dataset.labels = []
    })
    sortedDates.filter(date => date != null).forEach(function(date) {
        const converted_date = new Date(date)
        const values = filteredPlots.get(date)

        transactionChart.data.datasets[0].data.push({
            t: converted_date,
            y: values.reduce((acc, value) => acc + value.amount, 0.0).toFixed(2)
        })
        transactionChart.data.datasets[0].labels.push(values.map(function(value) {
            return value.description;
        }).join(", "))
    })

    transactionChart.update()
}
